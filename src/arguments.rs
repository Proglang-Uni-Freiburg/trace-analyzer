use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// The filepath to the trace file
    #[arg(short, long)]
    pub input: String,
}