use std::fs::read_to_string;
use clap::Parser;
use log::{error, info};
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
mod normalizer;

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

    let trace = match trace_grammar::parse(&tokens) {
        Ok(trace) => trace,
        Err(_) => {
            eprintln!("Parsing failed yikes");
            return;
        }
    };

    match analyzer::analyze_trace(&trace) {
        Ok(_) => {
            info!("Analyzer could not find a violation");
        }
        Err(error) => {
            error!("Analyzer found a violation in line {}: {}", error.line, error.error_type);
        }
    }
}
