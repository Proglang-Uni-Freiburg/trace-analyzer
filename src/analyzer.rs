use std::collections::{HashMap, HashSet};
use log::{debug};
use crate::error::{AnalyzerError, AnalyzerErrorType};
use crate::parser::{Event, Operand, Operation, Trace};

struct Lock<'a> {
    owner: Option<&'a str>,
    locked: bool,
}

pub fn analyze_trace<'a>(trace: &'a Trace) -> Result<(), AnalyzerError<'a>> {
    let mut locks: HashMap<&str, Lock> = HashMap::new();
    let mut memory_locations: HashSet<&str> = HashSet::new();
    let mut line = 1;

    for event in &trace.events {
        match event.operation {
            Operation::Acquire => {
                // 'acquire' operations only have 'lock_identifier' operands
                let lock_id = lock_id(&event, line)?;

                if let Some(lock) = locks.get(lock_id) {
                    if lock.locked {
                        let error = AnalyzerError {
                            line,
                            error_type: AnalyzerErrorType::RepeatedAcquisition {
                                lock_id,
                                thread_id: event.thread_identifier,
                            },
                        };
                        return Err(error);
                    }
                }

                let lock = Lock {
                    locked: true,
                    owner: Some(event.thread_identifier),
                };

                locks.insert(lock_id, lock);
                debug!("Thread '{}' acquired lock '{lock_id}' in line {line}", event.thread_identifier);
            }
            Operation::Release => {
                // 'release' operations only have 'lock_identifier' operands
                let lock_id = lock_id(&event, line)?;

                if let Some(lock) = locks.get(lock_id) {
                    if !lock.locked {
                        let error = AnalyzerError {
                            line,
                            error_type: AnalyzerErrorType::RepeatedRelease {
                                lock_id,
                                thread_id: event.thread_identifier,
                            },
                        };
                        return Err(error);
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
                            return Err(error);
                        }
                    }

                    let updated_lock = Lock {
                        locked: false,
                        owner: None,
                    };

                    locks.insert(lock_id, updated_lock);
                    debug!("Thread '{}' released lock '{lock_id}' in line {line}", event.thread_identifier);
                }
            }
            Operation::Write => {
                let memory_id = memory_id(&event, line)?;

                memory_locations.insert(memory_id);
                debug!("Thread '{}' wrote to memory location '{memory_id}' in line {line}", event.thread_identifier);
            }
            Operation::Read => {
                let memory_id = memory_id(&event, line)?;

                if let None = memory_locations.get(&memory_id) {
                    let error = AnalyzerError {
                        line,
                        error_type: AnalyzerErrorType::ReadFromUnwrittenMemory {
                            memory_id,
                            thread_id: event.thread_identifier,
                        },
                    };
                    return Err(error);
                }

                debug!("Thread '{}' read from memory location '{memory_id}' in line {line}", event.thread_identifier);
            }
            // other operations are not needed to check well-formedness
            _ => {}
        }
        line += 1;
    }
    Ok(())
}

fn lock_id<'a>(event: &'a Event, line: usize) -> Result<&'a str, AnalyzerError<'a>> {
    if let Operand::LockIdentifier(lock_identifier) = event.operand {
        Ok(lock_identifier)
    } else {
        let error = AnalyzerError {
            line,
            error_type: AnalyzerErrorType::MismatchedArguments {
                operation: event.clone().operation,
                operand: Operand::LockIdentifier("_"),
            },
        };
        Err(error)
    }
}

fn memory_id<'a>(event: &'a Event, line: usize) -> Result<&'a str, AnalyzerError<'a>> {
    if let Operand::MemoryLocation(memory_location) = event.operand {
        Ok(memory_location)
    } else {
        let error = AnalyzerError {
            line,
            error_type: AnalyzerErrorType::MismatchedArguments {
                operation: event.clone().operation,
                operand: Operand::MemoryLocation("_"),
            },
        };
        Err(error)
    }
}