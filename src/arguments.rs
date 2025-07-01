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
    /// If a graphical representation should be constructed (only suitable for small traces)
    #[arg(short, long)]
    pub graph: bool,
    /// Analyze trace via lock dependencies (only suitable for small traces)
    #[arg(short, long)]
    pub lock_dependencies: bool,
    /// If each violation should be logged individually (only suitable for small traces)
    #[arg(short, long)]
    pub verbose: bool,
}

impl Arguments {
    #[allow(dead_code)] // used when running tests
    pub fn new<S: Into<String>>(
        input: S,
        normalize: bool,
        graph: bool,
        lock_dependencies: bool,
        verbose: bool,
    ) -> Self {
        Self {
            input: input.into(),
            normalize,
            graph,
            lock_dependencies,
            verbose,
        }
    }
}
