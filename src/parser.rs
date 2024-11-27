use std::fmt::{Display, Formatter};
use peg::parser;
use crate::token::Token;

parser!(
    pub grammar trace_grammar<'a>() for [Token] {
        use crate::token::Token::*;

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

pub struct Trace {
    pub(crate) events: Vec<Event>,
}

impl Display for Trace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for event in &self.events {
            writeln!(f, "{}", event)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Event {
    pub(crate) thread_identifier: i64,
    pub(crate) operation: Operation,
    pub(crate) operand: Operand,
    loc: i64,
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event[Thread=T{} Operation={} Operand={} LoC={}]", self.thread_identifier, self.operation, self.operand, self.loc)
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
pub enum Operand {
    MemoryLocation(i64),
    LockIdentifier(i64),
    ThreadIdentifier(i64),
}

impl Operand {
    pub(crate) fn id(&self) -> i64 {
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
            Operand::MemoryLocation(memory_location) => write!(f, "M{memory_location}"),
            Operand::LockIdentifier(lock_identifier) => write!(f, "V{lock_identifier}"),
            Operand::ThreadIdentifier(thread_identifier) => write!(f, "T{thread_identifier}")
        }
    }
}