use std::fs::read_to_string;
use logos::{Logos};
use crate::parser::{analyzer};
use crate::token::Token;

mod token;
mod parser;

fn main() {
    let source = match read_to_string("input.txt") {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Reading the source file failed: {error}");
            return;
        }
    };

    let tokens = match Token::lexer(&source).collect::<Result<Vec<_>, ()>>() {
        Ok(tokens) => tokens,
        Err(_) => {
            eprintln!("Lexing failed yikes");
            return;
        }
    };

    match analyzer::parse(&tokens) {
        Ok(program) => {
            println!("{:?}", program)
        }
        Err(_) => {}
    }
}
