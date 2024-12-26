use crate::arguments::Arguments;
use crate::error::AnalyzerError;
use crate::lexer::tokenize_source;
use crate::parser::{parse_event, Operation};
use std::collections::{HashMap};
use std::fs::{File};
use std::io::{BufRead, BufReader};
use log::debug;

struct Lock {
    owner: Option<i64>,
    locked: bool,
}

pub fn analyze_trace(arguments: Arguments) -> Result<(), Vec<AnalyzerError>> {
    // store trace violations
    let mut errors = Vec::new();

    // stream content of file to avoid OOM
    let file_handle = match File::open(arguments.input) {
        Ok(file_handle) => file_handle,
        Err(err) => {
            errors.push(AnalyzerError::from(err));
            return Err(errors);
        }
    };

    let reader = BufReader::new(file_handle);

    let mut locks: HashMap<i64, Lock> = HashMap::new();
    let mut row = 1;

    for line in reader.lines() {
        let line = match line.map_err(AnalyzerError::from) {
            Ok(line) => line,
            Err(err) => {
                errors.push(AnalyzerError::from(err));
                return Err(errors);
            }
        };
        let tokens = match tokenize_source(line, arguments.normalize) {
            Ok(tokens) => tokens,
            Err(err) => {
                errors.push(err);
                return Err(errors);
            }
        };
        let event = match parse_event(tokens) {
            Ok(event) => event,
            Err(err) => {
                errors.push(err);
                return Err(errors);
            }
        };

        match event.operation {
            Operation::Acquire => {
                let lock_id = event.operand.id();

                if let Some(lock) = locks.get(&lock_id) {
                    if lock.locked {
                        let error = AnalyzerError::RepeatedAcquisition {
                            line: row,
                            lock_id,
                            thread_id: event.thread_identifier,
                        };
                        errors.push(error);
                        continue;
                    }
                }

                let lock = Lock {
                    locked: true,
                    owner: Some(event.thread_identifier),
                };

                locks.insert(lock_id, lock);
                debug!(
                    "Thread 'T{}' acquired lock 'L{lock_id}' in line {row}",
                    event.thread_identifier
                );
            }
            Operation::Release => {
                let lock_id = event.operand.id();

                match locks.get(&lock_id) {
                    None => {
                        let error = AnalyzerError::ReleasedNonAcquiredLock {
                            line: row,
                            lock_id,
                            thread_id: event.thread_identifier,
                        };
                        errors.push(error);
                        continue;
                    }
                    Some(lock) => {
                        if !lock.locked {
                            let error = AnalyzerError::RepeatedRelease {
                                line: row,
                                lock_id,
                                thread_id: event.thread_identifier,
                            };
                            errors.push(error);
                            continue;
                        }

                        if let Some(owner) = lock.owner {
                            if owner != event.thread_identifier {
                                let error = AnalyzerError::ReleasedNonOwningLock {
                                    line: row,
                                    lock_id,
                                    thread_id: event.thread_identifier,
                                    owner,
                                };
                                errors.push(error);
                                continue;
                            }
                        }

                        let updated_lock = Lock {
                            locked: false,
                            owner: None,
                        };

                        locks.insert(lock_id, updated_lock);
                        debug!(
                            "Thread 'T{}' released lock 'L{lock_id}' in line {row}",
                            event.thread_identifier
                        );
                    }
                }
            }
            // other operations are not needed to check well-formedness
            _ => {}
        }
        row += 1;
    }

    if errors.is_empty() {
        return Ok(());
    }

    Err(errors)
}

#[cfg(test)]
mod tests {
    use crate::analyzer::analyze_trace;
    use crate::arguments::Arguments;
    use crate::error::AnalyzerError;

    #[test]
    fn succeed_when_analyzing_valid_trace() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new("test/valid_trace.std", true);

        // act
        let result = analyze_trace(arguments);

        // assert
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn fail_when_acquire_lock_repeatedly() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new("test/repeated_lock_acquisition.std", true);

        // act
        let errors = analyze_trace(arguments).unwrap_err();

        // assert
        assert_eq!(errors.len(), 1);
        assert!(match errors[0] {
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
    fn fail_when_release_lock_repeatedly() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new("test/repeated_lock_release.std", true);

        // act
        let errors = analyze_trace(arguments).unwrap_err();

        // assert
        assert_eq!(errors.len(), 1);
        assert!(match errors[0] {
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
    fn fail_when_release_non_owning_lock() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new("test/release_non_owning_lock.std", true);

        // act
        let errors = analyze_trace(arguments).unwrap_err();

        // assert
        assert_eq!(errors.len(), 1);
        assert!(match errors[0] {
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
    fn fail_when_release_not_acquired_lock() -> Result<(), AnalyzerError> {
        // arrange
        let arguments = Arguments::new("test/release_non_acquired_lock.std", true);

        // act
        let errors = analyze_trace(arguments).unwrap_err();

        // assert
        assert_eq!(errors.len(), 1);
        assert!(match errors[0] {
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
}
