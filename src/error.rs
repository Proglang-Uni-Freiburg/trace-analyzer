use std::fmt::{Display, Formatter};
use crate::parser::{Operand, Operation};

pub struct AnalyzerError<'a> {
    pub(crate) line: usize,
    pub(crate) error_type: AnalyzerErrorType<'a>
}

pub(crate) enum AnalyzerErrorType<'a> {
    MismatchedArguments(&'a Operation, &'a Operand<'a>),
    RepeatedAcquisition(&'a str, &'a str)
}

impl Display for AnalyzerErrorType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzerErrorType::MismatchedArguments(operation, operand) => {
                write!(f, "Operation '{:?}' expected an operand of type '{:?}'", operation, operand)
            }
            AnalyzerErrorType::RepeatedAcquisition(lock_id, thread_identifier) => {
                write!(f, "Thread '{thread_identifier}' tried to acquire Lock '{lock_id}' which was already locked")
            }
        }
    }
}