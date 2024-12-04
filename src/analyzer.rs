use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{read_to_string};
use std::mem::discriminant;
use log::{debug};
use crate::arguments::Arguments;
use crate::error::{AnalyzerError, AnalyzerErrorType};
use crate::normalizer::normalize_tokens;
use crate::parser::{parse_tokens, Event, Operand, Operation};
use crate::token::{tokenize_source};

struct Lock {
    owner: Option<i64>,
    locked: bool,
}

pub fn analyze_trace(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    // read source file
    let input = read_to_string(arguments.input)?;

    // lex source file
    let tokens = tokenize_source(input)?;

    // normalize tokens if needed
    let tokens = if arguments.normalize { normalize_tokens(tokens) } else { tokens };

    // parse tokens
    let trace = parse_tokens(tokens)?;

    // analyze trace
    let mut locks: HashMap<i64, Lock> = HashMap::new();
    let mut memory_locations: HashSet<i64> = HashSet::new();
    let mut line = 1;

    for event in &trace.events {
        match event.operation {
            Operation::Acquire => {
                // 'acquire' operations only have 'lock_identifier' operands
                let lock_id = expect_operand(&event, &Operand::LockIdentifier(0), line)?;

                if let Some(lock) = locks.get(&lock_id) {
                    if lock.locked {
                        let error = AnalyzerError {
                            line,
                            error_type: AnalyzerErrorType::RepeatedAcquisition {
                                lock_id,
                                thread_id: event.thread_identifier,
                            },
                        };
                        return Err(Box::from(error));
                    }
                }

                let lock = Lock {
                    locked: true,
                    owner: Some(event.thread_identifier),
                };

                locks.insert(lock_id, lock);
                debug!("Thread 'T{}' acquired lock 'L{lock_id}' in line {line}", event.thread_identifier);
            }
            Operation::Release => {
                // 'release' operations only have 'lock_identifier' operands
                let lock_id = expect_operand(&event, &Operand::LockIdentifier(0), line)?;

                match locks.get(&lock_id) {
                    None => {
                        let error = AnalyzerError {
                            line,
                            error_type: AnalyzerErrorType::ReleasedNonAcquiredLock {
                                lock_id,
                                thread_id: event.thread_identifier,
                            },
                        };
                        return Err(Box::new(error));
                    }
                    Some(lock) => {
                        if !lock.locked {
                            let error = AnalyzerError {
                                line,
                                error_type: AnalyzerErrorType::RepeatedRelease {
                                    lock_id,
                                    thread_id: event.thread_identifier,
                                },
                            };
                            return Err(Box::new(error));
                        }

                        if let Some(owner) = lock.owner {
                            if owner != event.thread_identifier {
                                let error = AnalyzerError {
                                    line,
                                    error_type: AnalyzerErrorType::ReleasedNonOwningLock {
                                        lock_id,
                                        thread_id: event.thread_identifier,
                                        owner,
                                    },
                                };
                                return Err(Box::new(error));
                            }
                        }

                        let updated_lock = Lock {
                            locked: false,
                            owner: None,
                        };

                        locks.insert(lock_id, updated_lock);
                        debug!("Thread 'T{}' released lock 'L{lock_id}' in line {line}", event.thread_identifier);
                    }
                }
            }
            Operation::Write => {
                let memory_id = expect_operand(&event, &Operand::MemoryLocation(0), line)?;

                memory_locations.insert(memory_id);
                debug!("Thread 'T{}' wrote to memory location 'V{memory_id}' in line {line}", event.thread_identifier);
            }
            Operation::Read => {
                let memory_id = expect_operand(&event, &Operand::MemoryLocation(0), line)?;

                if memory_locations.get(&memory_id).is_none() {
                    let error = AnalyzerError {
                        line,
                        error_type: AnalyzerErrorType::ReadFromUnwrittenMemory {
                            memory_id,
                            thread_id: event.thread_identifier,
                        },
                    };
                    return Err(Box::new(error));
                }

                debug!("Thread 'T{}' read from memory location 'V{memory_id}' in line {line}", event.thread_identifier);
            }
            // other operations are not needed to check well-formedness
            _ => {}
        }
        line += 1;
    }
    Ok(())
}

fn expect_operand(event: &Event, operand: &Operand, line: usize) -> Result<i64, Box<dyn Error>> {
    if discriminant(&event.operand) == discriminant(operand) {
        return Ok(event.operand.id());
    }

    let error = AnalyzerError {
        line,
        error_type: AnalyzerErrorType::MismatchedArguments {
            operation: event.clone().operation,
            operand: operand.clone(),
        },
    };

    Err(Box::new(error))
}