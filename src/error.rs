use std::fmt::{Display, Formatter};
use crate::parser::{Operand, Operation};

pub struct AnalyzerError<'a> {
    pub(crate) line: usize,
    pub(crate) error_type: AnalyzerErrorType<'a>
}

pub(crate) enum AnalyzerErrorType<'a> {
    MismatchedArguments(&'a Operation, &'a Operand<'a>),
    // lock_id, thread_identifier
    RepeatedAcquisition(&'a str, &'a str),
    // lock_id, releasing_thread, owner
    DisallowedRelease(&'a str, &'a str, &'a str),
    // memory_id, thread_identifier
    ReadFromUnwrittenMemory(&'a str, &'a str),
}

impl Display for AnalyzerErrorType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzerErrorType::MismatchedArguments(operation, operand) => {
                write!(f, "Operation '{:?}' expected an operand of type '{:?}'", operation, operand)
            }
            AnalyzerErrorType::RepeatedAcquisition(lock_id, thread_identifier) => {
                write!(f, "Thread '{thread_identifier}' tried to acquire lock '{lock_id}' which was already locked")
            }
            AnalyzerErrorType::DisallowedRelease(lock_id, releasing_thread, owner) => {
                write!(f, "Thread '{releasing_thread}' tried to release lock '{lock_id}' which is owned by thread '{owner}'")
            }
            AnalyzerErrorType::ReadFromUnwrittenMemory(memory_id, thread_identifier) => {
                write!(f, "Thread {thread_identifier} tried to read from memory location '{memory_id}' which was not written to")
            }
        }
    }
}