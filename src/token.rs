use logos::Logos;

#[derive(Logos, Debug)]
#[logos(skip r"[ \r\t\n\f]+")]
pub enum Token<'a> {
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
    #[regex("T[0-9]+", |lex| lex.slice())]
    ThreadIdentifier(&'a str),
    #[regex("L[0-9]+", |lex| lex.slice())]
    LockIdentifier(&'a str),
    #[regex("V[0-9]+(\\.[0-9]+\\[[0-9]+\\])?", |lex| lex.slice())]
    MemoryLocation(&'a str),
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