use std::error::Error;
use std::fmt::{Display, Formatter};
use logos::{Lexer, Logos};

#[derive(Logos, Debug, Copy, Clone)]
#[logos(skip r"[ \r\t\n\f]+")]
#[logos(error = LexerError)]
pub(crate) enum Token {
    // single char tokens
    #[token("|")]
    Pipe,
    #[token("(")]
    LeftParenthesis,
    #[token(")")]
    RightParenthesis,
    #[token("[")]
    LeftSquareBracket,
    #[token("]")]
    RightSquareBracket,
    #[token("w")]
    Write,
    #[token("r")]
    Read,
    // multi char tokens
    #[regex("T[0-9]+", id)]
    ThreadIdentifier(i64),
    #[regex("L[0-9]+", id)]
    LockIdentifier(i64),
    #[regex("V[0-9]+(\\.[0-9]+\\[[0-9]+\\])?", id)]
    MemoryLocation(i64),
    #[token("fork")]
    Fork,
    #[token("req")]
    Request,
    #[token("acq")]
    Acquire,
    #[token("rel")]
    Release,
    #[token("join")]
    Join,
    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    LineNumber(i64),
}

fn id(lex: &mut Lexer<Token>) -> Option<i64> {
    let slice = lex.slice();
    let id = slice[1..slice.len()].parse().ok()?;

    Some(id)
}

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) enum LexerError {
    #[default]
    NonAsciiCharacter
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