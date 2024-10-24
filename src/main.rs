use std::fs::read_to_string;
use clap::Parser;
use logos::{Logos};
use crate::arguments::Arguments;
use crate::parser::{analyzer};
use crate::token::Token;

mod token;
mod parser;
mod arguments;

fn main() {
    let arguments = Arguments::parse();

    let input = match read_to_string(arguments.input) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("Reading the input file failed: {error}");
            return;
        }
    };

    let tokens = match Token::lexer(&input).collect::<Result<Vec<_>, ()>>() {
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
