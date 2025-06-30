use crate::error::AnalyzerError;
use crate::lexer::Token;
use peg::parser;
use std::fmt::{Display, Formatter};

parser!(
    pub grammar trace_grammar<'a>() for [Token] {
        use crate::lexer::Token::*;

        pub rule parse() -> Event
            = [ThreadIdentifier(thread_identifier)] [Pipe] operation:operation() [LeftParenthesis] operand:operand() [RightParenthesis] [Pipe] [LineNumber(loc)] {
                Event { thread_identifier, operation, operand, loc }
            }

        rule operation() -> Operation
            = [Read] { Operation::Read }
            / [Write] { Operation::Write }
            / [Acquire] { Operation::Acquire }
            / [Request] { Operation::Request }
            / [Release] { Operation::Release }
            / [Fork] { Operation::Fork }
            / [Join] { Operation::Join }

        rule operand() -> Operand
            = [MemoryLocation(memory_location)] { Operand::MemoryLocation(memory_location) }
            / [LockIdentifier(lock_identifier)] { Operand::LockIdentifier(lock_identifier) }
            / [ThreadIdentifier(thread_identifier)] { Operand::ThreadIdentifier(thread_identifier) }
    }
);

pub fn parse_event(tokens: Vec<Token>) -> Result<Event, AnalyzerError> {
    trace_grammar::parse(&tokens).map_err(AnalyzerError::from)
}

#[derive(Clone, PartialEq, Debug)]
pub struct Event {
    pub thread_identifier: i64,
    pub operation: Operation,
    pub operand: Operand,
    pub loc: i64,
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Event[Thread=T{} Operation={} Operand={} LoC={}]",
            self.thread_identifier, self.operation, self.operand, self.loc
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Read,
    Write,
    Acquire,
    Request,
    Release,
    Fork,
    Join,
    Begin,
    End,
    Branch,
}

impl Operation {
    pub fn new(value: i64) -> Option<Self> {
        match value {
            0 => Some(Operation::Acquire),
            1 => Some(Operation::Release),
            2 => Some(Operation::Read),
            3 => Some(Operation::Write),
            4 => Some(Operation::Fork),
            5 => Some(Operation::Join),
            6 => Some(Operation::Begin),
            7 => Some(Operation::End),
            8 => Some(Operation::Request),
            9 => Some(Operation::Branch),
            _ => None,
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Read => write!(f, "Read"),
            Operation::Write => write!(f, "Write"),
            Operation::Acquire => write!(f, "Acquire"),
            Operation::Request => write!(f, "Request"),
            Operation::Release => write!(f, "Release"),
            Operation::Fork => write!(f, "Fork"),
            Operation::Join => write!(f, "Join"),
            Operation::Begin => write!(f, "Begin"),
            Operation::End => write!(f, "End"),
            Operation::Branch => write!(f, "Branch"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    MemoryLocation(i64),
    LockIdentifier(i64),
    ThreadIdentifier(i64),
    None,
}

impl Operand {
    pub fn new(operation: &Operation, operand_id: i64) -> Self {
        match operation {
            Operation::Read => Operand::MemoryLocation(operand_id),
            Operation::Write => Operand::MemoryLocation(operand_id),
            Operation::Acquire => Operand::LockIdentifier(operand_id),
            Operation::Request => Operand::LockIdentifier(operand_id),
            Operation::Release => Operand::LockIdentifier(operand_id),
            Operation::Fork => Operand::ThreadIdentifier(operand_id),
            Operation::Join => Operand::ThreadIdentifier(operand_id),
            _ => Operand::None,
        }
    }

    pub fn id(&self) -> Option<i64> {
        match self {
            Operand::MemoryLocation(memory_id) => Some(*memory_id),
            Operand::LockIdentifier(lock_id) => Some(*lock_id),
            Operand::ThreadIdentifier(thread_id) => Some(*thread_id),
            Operand::None => None,
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::MemoryLocation(memory_location) => write!(f, "V{memory_location}"),
            Operand::LockIdentifier(lock_identifier) => write!(f, "L{lock_identifier}"),
            Operand::ThreadIdentifier(thread_identifier) => write!(f, "T{thread_identifier}"),
            Operand::None => write!(f, "None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize_source;
    use std::fs::read_to_string;

    #[test]
    fn succeed_when_parsing_valid_tokens() -> Result<(), AnalyzerError> {
        // arrange
        let input = read_to_string("test/valid_trace.std")?;
        let tokens = tokenize_source(input, true)?;

        // act
        let actual_event = parse_event(tokens)?;
        let expected_event = Event {
            thread_identifier: 6,
            operation: Operation::Write,
            operand: Operand::MemoryLocation(4294967298),
            loc: 59,
        };

        // assert
        assert_eq!(actual_event, expected_event);

        Ok(())
    }

    #[test]
    fn fail_when_parsing_invalid_tokens() -> Result<(), AnalyzerError> {
        // arrange
        let input = read_to_string("test/double_write_token.std")?;
        let tokens = tokenize_source(input, false)?;

        // act
        let error = parse_event(tokens).unwrap_err();

        // assert
        assert!(match error {
            AnalyzerError::ParserError(inner) => {
                assert_eq!(inner.location, 3);
                assert_eq!(
                    inner.expected.tokens().collect::<Vec<_>>(),
                    vec!["[LeftParenthesis]"]
                );

                true
            }
            _ => false,
        });

        Ok(())
    }
}
