use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct AnalyzerError {
    pub(crate) line: usize,
    pub(crate) error_type: AnalyzerErrorType,
}

impl Display for AnalyzerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Analyzer found a violation in line '{}': {}",
            self.line, self.error_type
        )
    }
}

impl Error for AnalyzerError {}

#[derive(Debug, PartialEq)]
pub enum AnalyzerErrorType {
    RepeatedAcquisition {
        lock_id: i64,
        thread_id: i64,
    },
    RepeatedRelease {
        lock_id: i64,
        thread_id: i64,
    },
    ReleasedNonOwningLock {
        lock_id: i64,
        thread_id: i64,
        owner: i64,
    },
    ReleasedNonAcquiredLock {
        lock_id: i64,
        thread_id: i64,
    },
    ReadFromUnwrittenMemory {
        memory_id: i64,
        thread_id: i64,
    },
}

impl Display for AnalyzerErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzerErrorType::RepeatedAcquisition { lock_id, thread_id } => {
                write!(f, "Thread 'T{thread_id}' tried to acquire lock 'L{lock_id}' which was already locked")
            }
            AnalyzerErrorType::ReleasedNonOwningLock {
                lock_id,
                thread_id,
                owner,
            } => {
                write!(f, "Thread 'T{thread_id}' tried to release lock 'L{lock_id}' which is owned by thread '{owner}'")
            }
            AnalyzerErrorType::ReadFromUnwrittenMemory {
                memory_id,
                thread_id,
            } => {
                write!(f, "Thread 'T{thread_id}' tried to read from memory location 'V{memory_id}' which was not written to")
            }
            AnalyzerErrorType::RepeatedRelease { lock_id, thread_id } => {
                write!(f, "Thread 'T{thread_id}' tried to release lock 'L{lock_id}' which was already released")
            }
            AnalyzerErrorType::ReleasedNonAcquiredLock { lock_id, thread_id } => {
                write!(f, "Thread 'T{thread_id}' tried to release lock 'L{lock_id}' which was not previously acquired")
            }
        }
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
                write!(f, "Logos encountered an non-ascii character")
            }
        }
    }
}

impl Error for LexerError {}
