use peg::parser;
use crate::token::Token;

parser!(
    pub grammar analyzer<'a>() for [Token<'a>] {
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
    traces: Vec<Trace<'a>>
}

#[derive(Debug)]
pub struct Trace<'a> {
    thread_identifier: &'a str,
    operation: Operation,
    operand: Operand<'a>,
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