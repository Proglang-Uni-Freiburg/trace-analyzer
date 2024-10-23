use std::fs::read_to_string;
use logos::{Logos};
use crate::token::Token;

mod token;

fn main() {
    let source = match read_to_string("input.txt") {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Reading the source file failed: {error}");
            return;
        }
    };

    let tokens = Token::lexer(&source).collect::<Vec<_>>();
    for token in tokens {
        match token {
            Ok(token) => {
                println!("{:?}", token);
            }
            Err(_) => {}
        }
    }
}
