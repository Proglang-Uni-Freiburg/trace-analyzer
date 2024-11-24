use std::fmt::{Display, Formatter};
use crate::parser::{Operand, Operation};

pub struct AnalyzerError<'a> {
    pub(crate) line: usize,
    pub(crate) error_type: AnalyzerErrorType<'a>,
}

pub(crate) enum AnalyzerErrorType<'a> {
    MismatchedArguments {
        operation: Operation,
        operand: Operand<'a>,
    },
    RepeatedAcquisition {
        lock_id: &'a str,
        thread_id: &'a str,
    },
    RepeatedRelease {
        lock_id: &'a str,
        thread_id: &'a str,
    },
    ReleasedNonOwningLock {
        lock_id: &'a str,
        thread_id: &'a str,
        owner: &'a str,
    },
    ReadFromUnwrittenMemory {
        memory_id: &'a str,
        thread_id: &'a str,
    },
}

impl Display for AnalyzerErrorType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzerErrorType::MismatchedArguments { operation, operand } => {
                write!(f, "Operation '{operation}' expected an operand of type '{operand}'")
            }
            AnalyzerErrorType::RepeatedAcquisition { lock_id, thread_id } => {
                write!(f, "Thread '{thread_id}' tried to acquire lock '{lock_id}' which was already locked")
            }
            AnalyzerErrorType::ReleasedNonOwningLock { lock_id, thread_id, owner } => {
                write!(f, "Thread '{thread_id}' tried to release lock '{lock_id}' which is owned by thread '{owner}'")
            }
            AnalyzerErrorType::ReadFromUnwrittenMemory { memory_id, thread_id } => {
                write!(f, "Thread '{thread_id}' tried to read from memory location '{memory_id}' which was not written to")
            }
            AnalyzerErrorType::RepeatedRelease { lock_id, thread_id } => {
                write!(f, "Thread '{thread_id}' tried to release lock '{lock_id}' which was already released")
            }
        }
    }
}