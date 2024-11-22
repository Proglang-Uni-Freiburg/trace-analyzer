use peg::parser;
use crate::token::Token;

parser!(
    pub grammar trace_grammar<'a>() for [Token<'a>] {
        use crate::token::Token::*;

        pub rule parse() -> Program<'a>
            = traces:trace()* {
                Program { traces }
            }

        rule trace() -> Trace<'a>
            = [ThreadIdentifier(thread_identifier)] [Pipe] operation:operation() [LeftParenthesis] operand:operand() [RightParenthesis] [Pipe] [LineNumber(loc)] {
                Trace { thread_identifier, operation, operand, loc }
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

#[derive(Debug)]
pub struct Program<'a> {
    pub(crate) traces: Vec<Trace<'a>>
}

#[derive(Debug)]
pub struct Trace<'a> {
    pub(crate) thread_identifier: &'a str,
    pub(crate) operation: Operation,
    pub(crate) operand: Operand<'a>,
    loc: i64,
}

#[derive(Debug)]
pub enum Operation {
    Read,
    Write,
    Acquire,
    Request,
    Release,
    Fork,
    Join
}

#[derive(Debug)]
pub enum Operand<'a> {
    MemoryLocation(&'a str),
    LockIdentifier(&'a str),
    ThreadIdentifier(&'a str),
}

/*
We restrict our attention to well-formed traces 𝜎, that abide to shared-memory semantics. That is,
if a lock ℓ is acquired at an event 𝑒 by thread 𝑡, then any later acquisition event 𝑒′ of the same lock
ℓ must be preceded by an event 𝑒′′ that releases lock ℓ in thread 𝑡 in between the occurrence of 𝑒
and 𝑒′. Taking 𝑒′′ to be the earliest such release event, we say that 𝑒 and 𝑒′′ are matching acquire
and release events, and denote this by 𝑒 = match𝜎 (𝑒′′) and 𝑒′′ = match𝜎 (𝑒). Moreover, every read
event has at least one preceding write event on the same location, that it reads its value from.
(Page 5 Paper)
 */