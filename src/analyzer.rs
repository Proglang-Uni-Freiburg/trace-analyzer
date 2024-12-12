use crate::arguments::Arguments;
use crate::error::AnalyzerError;
use crate::lexer::tokenize_source;
use crate::parser::{parse_tokens, Operation};
use log::debug;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;

struct Lock {
    owner: Option<i64>,
    locked: bool,
}

pub fn analyze_trace(arguments: Arguments) -> Result<(), AnalyzerError> {
    // read source file
    let input = match read_to_string(arguments.input) {
        Ok(input) => input,
        Err(error) => {
            return Err(AnalyzerError::from(error));
        }
    };

    // lex source file
    let tokens = match tokenize_source(input, arguments.normalize) {
        Ok(tokens) => tokens,
        Err(error) => return Err(AnalyzerError::from(error)),
    };

    // parse tokens
    let trace = match parse_tokens(tokens) {
        Ok(trace) => trace,
        Err(error) => return Err(AnalyzerError::from(error)),
    };

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
                        let error = AnalyzerError::RepeatedAcquisition {
                            line,
                            lock_id,
                            thread_id: event.thread_identifier,
                        };
                        return Err(error);
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
                        let error = AnalyzerError::ReleasedNonAcquiredLock {
                            line,
                            lock_id,
                            thread_id: event.thread_identifier,
                        };
                        return Err(error);
                    }
                    Some(lock) => {
                        if !lock.locked {
                            let error = AnalyzerError::RepeatedRelease {
                                line,
                                lock_id,
                                thread_id: event.thread_identifier,
                            };
                            return Err(error);
                        }

                        if let Some(owner) = lock.owner {
                            if owner != event.thread_identifier {
                                let error = AnalyzerError::ReleasedNonOwningLock {
                                    line,
                                    lock_id,
                                    thread_id: event.thread_identifier,
                                    owner,
                                };
                                return Err(error);
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
                    let error = AnalyzerError::ReadFromUnwrittenMemory {
                        line,
                        memory_id,
                        thread_id: event.thread_identifier,
                    };
                    return Err(error);
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
    use crate::arguments::Arguments;
    use crate::error::AnalyzerError;
    use std::error::Error;

    #[test]
    fn succeed_when_analyzing_valid_trace() -> Result<(), Box<dyn Error>> {
        // arrange
        let arguments = Arguments::new("test/valid_trace.std", true);

        // act
        let result = analyze_trace(arguments);

        // assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn fail_when_acquire_lock_repeatedly() -> Result<(), Box<dyn Error>> {
        // arrange
        let arguments = Arguments::new("test/repeated_lock_acquisition.std", true);

        // act
        let error = analyze_trace(arguments).unwrap_err();

        // assert
        assert!(match error {
            AnalyzerError::RepeatedAcquisition {
                line,
                lock_id,
                thread_id,
            } => {
                assert_eq!(line, 7);
                assert_eq!(lock_id, 9);
                assert_eq!(thread_id, 7);

                true
            }
            _ => false,
        });

        Ok(())
    }

    #[test]
    fn fail_when_release_lock_repeatedly() -> Result<(), Box<dyn Error>> {
        // arrange
        let arguments = Arguments::new("test/repeated_lock_release.std", true);

        // act
        let error = analyze_trace(arguments).unwrap_err();

        // assert

        assert!(match error {
            AnalyzerError::RepeatedRelease {
                line,
                lock_id,
                thread_id,
            } => {
                assert_eq!(line, 8);
                assert_eq!(lock_id, 9);
                assert_eq!(thread_id, 6);

                true
            }
            _ => false,
        });

        Ok(())
    }

    #[test]
    fn fail_when_release_non_owning_lock() -> Result<(), Box<dyn Error>> {
        // arrange
        let arguments = Arguments::new("test/release_non_owning_lock.std", true);

        // act
        let error = analyze_trace(arguments).unwrap_err();

        // assert
        assert!(match error {
            AnalyzerError::ReleasedNonOwningLock {
                line,
                lock_id,
                thread_id,
                owner,
            } => {
                assert_eq!(line, 7);
                assert_eq!(lock_id, 9);
                assert_eq!(thread_id, 7);
                assert_eq!(owner, 6);

                true
            }
            _ => false,
        });

        Ok(())
    }

    #[test]
    fn fail_when_release_not_acquired_lock() -> Result<(), Box<dyn Error>> {
        // arrange
        let arguments = Arguments::new("test/release_non_acquired_lock.std", true);

        // act
        let error = analyze_trace(arguments).unwrap_err();

        // assert
        assert!(match error {
            AnalyzerError::ReleasedNonAcquiredLock {
                line,
                lock_id,
                thread_id,
            } => {
                assert_eq!(line, 6);
                assert_eq!(lock_id, 9);
                assert_eq!(thread_id, 7);

                true
            }
            _ => false,
        });

        Ok(())
    }

    #[test]
    fn fail_when_read_from_unwritten_memory() -> Result<(), Box<dyn Error>> {
        // arrange
        let arguments = Arguments::new("test/read_from_unwritten_memory.std", true);

        // act
        let error = analyze_trace(arguments).unwrap_err();

        // assert
        assert!(match error {
            AnalyzerError::ReadFromUnwrittenMemory {
                line,
                memory_id,
                thread_id,
            } => {
                assert_eq!(line, 1);
                assert_eq!(memory_id, 4294967298);
                assert_eq!(thread_id, 6);

                true
            }
            _ => false,
        });

        Ok(())
    }
}
