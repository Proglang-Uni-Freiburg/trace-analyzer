use std::collections::HashMap;
use log::{debug};
use crate::error::{AnalyzerError, AnalyzerErrorType};
use crate::parser::{Operand, Operation, Trace};

struct Lock<'a> {
    id: &'a str,
    accessor: &'a str,
    is_locked: bool,
}

#[allow(dead_code)]
struct MemoryLocation<'a> {
    id: &'a str,
}

pub fn analyze_trace<'a>(trace: &'a Trace) -> Result<(), AnalyzerError<'a>> {
    let mut lock_map: HashMap<&str, Lock> = HashMap::new();
    let mut memory_map: HashMap<&str, MemoryLocation> = HashMap::new();
    let mut line = 1;

    for event in &trace.events {
        match event.operation {
            Operation::Acquire => {
                // 'acquire' operations only have 'lock_identifier' operands
                let lock_id = lock_id(&event.operand, &event.operation, line)?;

                if let Some(lock) = lock_map.get(lock_id) {
                    // repeated acquisition of the same lock
                    if lock.is_locked {
                        let error = AnalyzerError {
                            line,
                            error_type: AnalyzerErrorType::RepeatedAcquisition(lock.id, event.thread_identifier),
                        };
                        return Err(error);
                    }
                }

                let lock = Lock {
                    id: lock_id,
                    is_locked: true,
                    accessor: event.thread_identifier,
                };

                lock_map.insert(lock_id, lock);
                debug!("Locked lock '{lock_id}' by thread '{}' in line {line}", event.thread_identifier);
            }
            Operation::Release => {
                // 'release' operations only have 'lock_identifier' operands
                let lock_id = lock_id(&event.operand, &event.operation, line)?;

                // TODO is releasing an unlocked lock a violation?

                if let Some(lock_state) = lock_map.get(lock_id) {
                    // repeated acquisition of the same lock
                    if lock_state.is_locked {
                        if lock_state.accessor != event.thread_identifier {
                            let error = AnalyzerError {
                                line,
                                error_type: AnalyzerErrorType::DisallowedRelease(lock_state.id, event.thread_identifier, lock_state.accessor),
                            };
                            return Err(error);
                        }

                        let updated_lock = Lock {
                            id: lock_id,
                            is_locked: false,
                            accessor: event.thread_identifier,
                        };

                        lock_map.insert(lock_id, updated_lock);
                        debug!("Unlocked lock '{lock_id}' by thread '{}' in line {line}", event.thread_identifier);
                    }
                }
            }
            Operation::Write => {
                let memory_id = memory_id(&event.operand, &event.operation, line)?;
                let memory_location = MemoryLocation {
                    id: memory_id,
                };

                memory_map.insert(memory_id, memory_location);
                debug!("Thread '{}' wrote to memory location '{memory_id}' in line {line}", event.thread_identifier);
            }
            Operation::Read => {
                let memory_id = memory_id(&event.operand, &event.operation, line)?;

                if let None = memory_map.get(memory_id) {
                    let error = AnalyzerError {
                        line,
                        error_type: AnalyzerErrorType::ReadFromUnwrittenMemory(memory_id, event.thread_identifier),
                    };
                    return Err(error);
                } else {
                    debug!("Thread '{}' read from memory location '{memory_id}' in line {line}", event.thread_identifier);
                }
            }
            // other operations are not needed to check well-formedness
            _ => {}
        }

        line += 1;
    }
    Ok(())
}

fn lock_id<'a>(operand: &'a Operand, operation: &'a Operation, line: usize) -> Result<&'a str, AnalyzerError<'a>> {
    if let Operand::LockIdentifier(lock_identifier) = operand {
        Ok(lock_identifier)
    } else {
        let error = AnalyzerError {
            line,
            error_type: AnalyzerErrorType::MismatchedArguments(operation, &Operand::LockIdentifier("_")),
        };
        Err(error)
    }
}

fn memory_id<'a>(operand: &'a Operand, operation: &'a Operation, line: usize) -> Result<&'a str, AnalyzerError<'a>> {
    if let Operand::MemoryLocation(memory_location) = operand {
        Ok(memory_location)
    } else {
        let error = AnalyzerError {
            line,
            error_type: AnalyzerErrorType::MismatchedArguments(operation, &Operand::MemoryLocation("_")),
        };
        Err(error)
    }
}