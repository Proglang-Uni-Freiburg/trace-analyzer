use crate::lexer::Token;
use peg::parser;
use std::error::Error;
use std::fmt::{Display, Formatter};

parser!(
    pub grammar trace_grammar<'a>() for [Token] {
        use crate::lexer::Token::*;

        pub rule parse() -> Trace
            = events:event()* {
                Trace { events }
            }

        rule event() -> Event
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

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Trace, Box<dyn Error>> {
    match trace_grammar::parse(&tokens) {
        Ok(trace) => Ok(trace),
        Err(error) => Err(Box::new(error)),
    }
}

#[derive(Debug)]
pub struct Trace {
    pub events: Vec<Event>,
}

impl Display for Trace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for event in &self.events {
            writeln!(f, "{}", event)?;
        }
        Ok(())
    }
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
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    MemoryLocation(i64),
    LockIdentifier(i64),
    ThreadIdentifier(i64),
}

impl Operand {
    pub fn id(&self) -> i64 {
        match self {
            Operand::MemoryLocation(memory_id) => *memory_id,
            Operand::LockIdentifier(lock_id) => *lock_id,
            Operand::ThreadIdentifier(thread_id) => *thread_id,
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::MemoryLocation(memory_location) => write!(f, "V{memory_location}"),
            Operand::LockIdentifier(lock_identifier) => write!(f, "L{lock_identifier}"),
            Operand::ThreadIdentifier(thread_identifier) => write!(f, "T{thread_identifier}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize_source;
    use std::fs::read_to_string;

    #[test]
    fn succeed_when_parsing_valid_tokens() -> Result<(), Box<dyn Error>> {
        let input = read_to_string("test/valid_trace.std")?;
        let tokens = tokenize_source(input, true)?;

        let expected_event = Event {
            thread_identifier: 6,
            operation: Operation::Write,
            operand: Operand::MemoryLocation(4294967298),
            loc: 59,
        };

        let result = parse_tokens(tokens);
        assert!(result.is_ok());

        let trace = result?;
        assert_eq!(trace.events.len(), 7);
        assert_eq!(trace.events[0], expected_event);

        Ok(())
    }

    #[test]
    fn fail_when_parsing_invalid_tokens() -> Result<(), Box<dyn Error>> {
        let input = read_to_string("test/double_write_token.std")?;
        let tokens = tokenize_source(input, false)?;

        let result = parse_tokens(tokens);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.to_string(), "error at 3: expected [LeftParenthesis]");

        Ok(())
    }
}
