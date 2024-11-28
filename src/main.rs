use clap::Parser;
use log::{error, info};
use simple_logger::SimpleLogger;
use arguments::Arguments;

mod token;
mod parser;
mod arguments;
mod analyzer;
mod error;
mod normalizer;

fn main() {
    SimpleLogger::new().init().unwrap();
    let arguments = Arguments::parse();

    match analyzer::analyze_trace(&arguments) {
        Ok(_) => info!("Analyzer could not find a violation"),
        Err(error) => error!("{error}"),
    }
}
