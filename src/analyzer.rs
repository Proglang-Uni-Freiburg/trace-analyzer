use crate::error::{AnalyzerError, AnalyzerErrorType};
use crate::lexer::tokenize_source;
use crate::parser::{parse_tokens, Operation};
use log::debug;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;

struct Lock {
    owner: Option<i64>,
    locked: bool,
}

pub fn analyze_trace<S: AsRef<Path>>(input: S, normalize: bool) -> Result<(), Box<dyn Error>> {
    // read source file
    let input = read_to_string(input)?;

    // lex source file
    let tokens = tokenize_source(input, normalize)?;

    // parse tokens
    let trace = parse_tokens(tokens)?;

    // analyze trace
    let mut locks: HashMap<i64, Lock> = HashMap::new();
    let mut memory_locations: HashSet<i64> = HashSet::new();
    let mut line = 1;

    for event in &trace.events {
        match event.operation {
            Operation::Acquire => {
                let lock_id = event.operand.id();

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
                debug!(
                    "Thread 'T{}' acquired lock 'L{lock_id}' in line {line}",
                    event.thread_identifier
                );
            }
            Operation::Release => {
                let lock_id = event.operand.id();

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
                        debug!(
                            "Thread 'T{}' released lock 'L{lock_id}' in line {line}",
                            event.thread_identifier
                        );
                    }
                }
            }
            Operation::Write => {
                let memory_id = event.operand.id();

                memory_locations.insert(memory_id);
                debug!(
                    "Thread 'T{}' wrote to memory location 'V{memory_id}' in line {line}",
                    event.thread_identifier
                );
            }
            Operation::Read => {
                let memory_id = event.operand.id();

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

                debug!(
                    "Thread 'T{}' read from memory location 'V{memory_id}' in line {line}",
                    event.thread_identifier
                );
            }
            // other operations are not needed to check well-formedness
            _ => {}
        }
        line += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::analyzer::analyze_trace;
    use crate::error::{AnalyzerError, AnalyzerErrorType};
    use std::error::Error;

    #[test]
    fn succeed_when_analyzing_valid_trace() -> Result<(), Box<dyn Error>> {
        let result = analyze_trace("test/valid_trace.std", true);

        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn fail_when_acquire_lock_repeatedly() -> Result<(), Box<dyn Error>> {
        let result = analyze_trace("test/repeated_lock_acquisition.std", true);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<AnalyzerError>()?;
        assert_eq!(error.line, 7);
        assert_eq!(
            error.error_type,
            AnalyzerErrorType::RepeatedAcquisition {
                lock_id: 9,
                thread_id: 7
            }
        );

        Ok(())
    }

    #[test]
    fn fail_when_release_lock_repeatedly() -> Result<(), Box<dyn Error>> {
        let result = analyze_trace("test/repeated_lock_release.std", true);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<AnalyzerError>()?;
        assert_eq!(error.line, 8);
        assert_eq!(
            error.error_type,
            AnalyzerErrorType::RepeatedRelease {
                lock_id: 9,
                thread_id: 6
            }
        );

        Ok(())
    }

    #[test]
    fn fail_when_release_non_owning_lock() -> Result<(), Box<dyn Error>> {
        let result = analyze_trace("test/release_non_owning_lock.std", true);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<AnalyzerError>()?;
        assert_eq!(error.line, 7);
        assert_eq!(
            error.error_type,
            AnalyzerErrorType::ReleasedNonOwningLock {
                owner: 6,
                lock_id: 9,
                thread_id: 7
            }
        );

        Ok(())
    }

    #[test]
    fn fail_when_release_not_acquired_lock() -> Result<(), Box<dyn Error>> {
        let result = analyze_trace("test/release_non_acquired_lock.std", true);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<AnalyzerError>()?;
        assert_eq!(error.line, 6);
        assert_eq!(
            error.error_type,
            AnalyzerErrorType::ReleasedNonAcquiredLock {
                lock_id: 9,
                thread_id: 7
            }
        );

        Ok(())
    }

    #[test]
    fn fail_when_read_from_unwritten_memory() -> Result<(), Box<dyn Error>> {
        let result = analyze_trace("test/read_from_unwritten_memory.std", true);
        assert!(result.is_err());

        let error = result.unwrap_err().downcast::<AnalyzerError>()?;
        assert_eq!(error.line, 1);
        assert_eq!(
            error.error_type,
            AnalyzerErrorType::ReadFromUnwrittenMemory {
                memory_id: 4294967298,
                thread_id: 6
            }
        );

        Ok(())
    }
}
