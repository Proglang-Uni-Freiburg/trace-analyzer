use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// The filepath to the trace file
    #[arg(short, long)]
    pub input: String,
    /// If the input file should be normalized
    #[arg(short, long)]
    pub normalize: bool,
}

impl Arguments {
    #[allow(dead_code)] // used when running tests
    pub fn new<S: Into<String>>(input: S, normalize: bool) -> Self {
        Self {
            input: input.into(),
            normalize,
        }
    }
}
