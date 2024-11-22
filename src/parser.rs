use std::fmt::{Display, Formatter};
use peg::parser;
use crate::token::Token;

parser!(
    pub grammar trace_grammar<'a>() for [Token<'a>] {
        use crate::token::Token::*;

        pub rule parse() -> Trace<'a>
            = events:event()* {
                Trace { events }
            }

        rule event() -> Event<'a>
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

        rule operand() -> Operand<'a>
            = [MemoryLocation(memory_location)] { Operand::MemoryLocation(memory_location) }
            / [LockIdentifier(lock_identifier)] { Operand::LockIdentifier(lock_identifier) }
            / [ThreadIdentifier(thread_identifier)] { Operand::ThreadIdentifier(thread_identifier) }
    }
);

pub struct Trace<'a> {
    pub(crate) events: Vec<Event<'a>>
}

impl Display for Trace<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for event in &self.events {
            writeln!(f, "{}", event)?;
        }
        Ok(())
    }
}

pub struct Event<'a> {
    pub(crate) thread_identifier: &'a str,
    pub(crate) operation: Operation,
    pub(crate) operand: Operand<'a>,
    loc: i64,
}

impl Display for Event<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event[Thread={} Operation={} Operand={} LoC={}]", self.thread_identifier, self.operation, self.operand, self.loc)
    }
}

pub enum Operation {
    Read,
    Write,
    Acquire,
    Request,
    Release,
    Fork,
    Join
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

pub enum Operand<'a> {
    MemoryLocation(&'a str),
    LockIdentifier(&'a str),
    ThreadIdentifier(&'a str),
}

impl Display for Operand<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::MemoryLocation(memory_location) => write!(f, "{memory_location}"),
            Operand::LockIdentifier(lock_identifier) => write!(f, "{lock_identifier}"),
            Operand::ThreadIdentifier(thread_identifier) => write!(f, "{thread_identifier}")
        }
    }
}