use peg::error::ParseError;
use std::fmt::{Debug, Display, Formatter};
use std::io::Error as IOError;

#[derive(Debug)]
pub enum AnalyzerError {
    RepeatedAcquisition {
        line: usize,
        lock_id: i64,
        thread_id: i64,
    },
    RepeatedRelease {
        line: usize,
        lock_id: i64,
        thread_id: i64,
    },
    ReleasedNonOwningLock {
        line: usize,
        lock_id: i64,
        thread_id: i64,
        owner: i64,
    },
    ReleasedNonAcquiredLock {
        line: usize,
        lock_id: i64,
        thread_id: i64,
    },
    // wrapped errors
    IOError(IOError),
    LexerError(LexerError),
    ParserError(ParseError<usize>),
}

impl Display for AnalyzerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let description = match self {
            AnalyzerError::RepeatedAcquisition {
                line,
                lock_id,
                thread_id,
            } => {
                format!("Analyzer found a violation in line {line}: Thread 'T{thread_id}' tried to acquire lock 'L{lock_id}' which was already locked")
            }
            AnalyzerError::RepeatedRelease {
                line,
                lock_id,
                thread_id,
            } => {
                format!("Analyzer found a violation in line {line}: Thread 'T{thread_id}' tried to release lock 'L{lock_id}' which was already released")
            }
            AnalyzerError::ReleasedNonOwningLock {
                line,
                lock_id,
                thread_id,
                owner,
            } => {
                format!("Analyzer found a violation in line {line}: Thread 'T{thread_id}' tried to release lock 'L{lock_id}' which is owned by thread '{owner}'")
            }
            AnalyzerError::ReleasedNonAcquiredLock {
                line,
                lock_id,
                thread_id,
            } => {
                format!("Analyzer found a violation in line {line}: Thread 'T{thread_id}' tried to release lock 'L{lock_id}' which was not previously acquired")
            }
            AnalyzerError::IOError(error) => {
                format!(
                    "Analyzer encountered an error while performing an I/O operation: {}",
                    error
                )
            }
            AnalyzerError::LexerError(error) => {
                format!("Analyzer encountered an error while lexing: {}", error)
            }
            AnalyzerError::ParserError(error) => {
                format!(
                    "Analyzer encountered a parser error at location '{}': {}",
                    error.location, error.expected
                )
            }
        };

        write!(f, "{}", description)
    }
}

impl From<LexerError> for AnalyzerError {
    fn from(error: LexerError) -> Self {
        AnalyzerError::LexerError(error)
    }
}

impl From<IOError> for AnalyzerError {
    fn from(error: IOError) -> Self {
        AnalyzerError::IOError(error)
    }
}

impl From<ParseError<usize>> for AnalyzerError {
    fn from(error: ParseError<usize>) -> Self {
        AnalyzerError::ParserError(error)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexerError {
    #[default]
    NonAsciiCharacter,
}

impl Display for LexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerError::NonAsciiCharacter => {
                write!(f, "Logos encountered a non-ascii character")
            }
        }
    }
}
