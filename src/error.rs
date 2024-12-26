use peg::error::ParseError;
use std::fmt::{Debug, Display, Formatter};
use std::io::Error as IOError;

#[derive(Debug)]
pub enum AnalyzerError {
    RepeatedAcquisition {
        attempted: usize,
        previous: usize,
        lock_id: i64,
        thread_id: i64,
    },
    RepeatedRelease {
        attempted: usize,
        previous: usize,
        lock_id: i64,
        thread_id: i64,
    },
    ReleasedNonOwningLock {
        row: usize,
        lock_id: i64,
        thread_id: i64,
        owner: i64,
    },
    ReleasedNonAcquiredLock {
        row: usize,
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
                attempted,
                previous,
                lock_id,
                thread_id,
            } => {
                format!("Thread 'T{thread_id}' tried to acquire the already acquired lock 'L{lock_id}' in row {attempted}. Last acquisition occurred in row {previous}")
            }
            AnalyzerError::RepeatedRelease {
                attempted,
                previous,
                lock_id,
                thread_id,
            } => {
                format!("Thread 'T{thread_id}' tried to release the already released lock 'L{lock_id}' in row {attempted}. Last release occurred in row {previous}")
            }
            AnalyzerError::ReleasedNonOwningLock {
                row,
                lock_id,
                thread_id,
                owner,
            } => {
                format!("Thread 'T{thread_id}' tried to release the non-owning lock 'L{lock_id}' in row {row}. Current owner is thread '{owner}'")
            }
            AnalyzerError::ReleasedNonAcquiredLock {
                row,
                lock_id,
                thread_id,
            } => {
                format!("Thread 'T{thread_id}' tried to release the non-acquired lock 'L{lock_id}' in row {row}")
            }
            AnalyzerError::IOError(error) => {
                format!("Analyzer encountered an error while performing I/O: {}", error)
            }
            AnalyzerError::LexerError(error) => {
                format!("Lexer encountered an error: {}", error)
            }
            AnalyzerError::ParserError(error) => {
                format!(
                    "Parser encountered an error at index {}: Expected {}",
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
                write!(f, "Could not lex non-ascii character")
            }
        }
    }
}
