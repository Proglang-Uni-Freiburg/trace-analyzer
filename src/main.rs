use arguments::Arguments;
use clap::Parser;
use log::{error, info};

mod analyzer;
mod arguments;
mod error;
mod lexer;
mod normalizer;
mod parser;

fn main() {
    env_logger::init();
    let arguments = Arguments::parse();

    match analyzer::analyze_trace(&arguments) {
        Ok(_) => info!("Analyzer could not find a violation"),
        Err(errors) => {
            error!(
                "Analyzer found {} errors in the analyzed trace",
                errors.len()
            );

            if &arguments.verbose == &true {
                for error in errors {
                    error!("{}", error);
                }
            }
        }
    }
}
