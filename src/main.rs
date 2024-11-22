use std::fs::read_to_string;
use clap::Parser;
use log::error;
use logos::{Logos};
use simple_logger::SimpleLogger;
use crate::parser::{trace_grammar};
use crate::arguments::Arguments;
use crate::token::Token;

mod token;
mod parser;
mod arguments;
mod analyzer;
mod error;

fn main() {
    SimpleLogger::new().init().unwrap();
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

    let program = match trace_grammar::parse(&tokens) {
        Ok(program) => program,
        Err(_) => {
            eprintln!("Parsing failed yikes");
            return;
        }
    };

    match analyzer::analyze_program(&program) {
        Ok(_) => {}
        Err(error) => {
            error!("Analyzer found a violation in line {}: {}", error.line, error.error_type);
        }
    }
}
