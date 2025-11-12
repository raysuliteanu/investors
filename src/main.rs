use std::{
    error::Error,
    fs::File,
    io::{self, Write},
};

use clap::{Args, Parser, Subcommand};
use investors::av::AV;

#[derive(Debug, Parser)]
struct Cli {
    /// specify API key; if missing, taken from the environment variable ALPHAVANTAGE_API_KEY
    #[arg(short, long)]
    key: Option<String>,
    /// write the output to a file instead of stdout
    #[arg(short, long)]
    output: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Quote(QuoteArgs),
}

#[derive(Debug, Args, Default)]
struct QuoteArgs {
    #[arg(name = "symbol")]
    symbol: String,
}

fn get_file(name: Option<String>) -> Result<Box<dyn Write>, io::Error> {
    let file: Box<dyn Write> = if let Some(output) = name {
        Box::new(File::create(output)?)
    } else {
        Box::new(io::stdout())
    };

    Ok(file)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    // TODO: if key not on cmd line and not in env, then notify user and exit
    let key = args
        .key
        .unwrap_or_else(|| std::env::var("ALPHAVANTAGE_API_KEY").unwrap());

    let av = AV::new(key);
    let mut out = get_file(args.output)?;
    match args.command {
        Commands::Quote(args) => {
            let quote = av.quote(&args.symbol).await?;
            write!(out, "{quote}")?;
            Ok(())
        }
    }
}
