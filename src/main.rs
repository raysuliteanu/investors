use std::{
    error::Error,
    fs::File,
    io::{self, Write},
    sync::Arc,
};

use clap::{Args, Parser, Subcommand};
use investors::av::AV;
use log::debug;
use tokio::task::JoinSet;

#[derive(Debug, Parser)]
struct Cli {
    /// specify API key; if missing, taken from the environment variable ALPHAVANTAGE_API_KEY
    #[arg(short, long)]
    key: Option<String>,
    /// write the output to a file instead of stdout
    #[arg(short, long)]
    output: Option<String>,
    /// output in csv (default is json)
    #[arg(long)]
    csv: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// get a stock quote
    Quote(QuoteArgs),
    /// get historical daily stock quote data
    Daily(DailyQuoteArgs),
}

#[derive(Debug, Args, Default)]
struct QuoteArgs {
    /// stock ticker symbol
    #[arg(name = "symbol")]
    symbol: Vec<String>,
}

#[derive(Debug, Args, Default)]
struct DailyQuoteArgs {
    /// if set, retrieve all data points; default is compact (latest 100 data points)
    #[arg(long)]
    full: bool,
    /// stock ticker symbol
    #[arg(name = "symbol")]
    symbol: Vec<String>,
}

fn get_file(name: Option<String>) -> Result<Box<dyn Write>, io::Error> {
    let file: Box<dyn Write> = if let Some(output) = name {
        Box::new(File::create(output)?)
    } else {
        Box::new(io::stdout().lock())
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

    let av = Arc::new(AV::new(key, args.csv));
    let mut out = get_file(args.output)?;
    match args.command {
        Commands::Quote(args) => {
            let mut tasks = JoinSet::new();

            for ticker in &args.symbol {
                let ticker = ticker.clone();
                let av = Arc::clone(&av);
                tasks.spawn(async move {
                    debug!("Getting quote for symbol: {ticker}");
                    (ticker.clone(), av.quote(&ticker).await)
                });
            }

            while let Some(result) = tasks.join_next().await {
                match result {
                    Ok((_ticker, Ok(quote))) => {
                        writeln!(out, "{quote}")?;
                    }
                    Ok((ticker, Err(e))) => {
                        eprintln!("Error getting quote for {}: {}", ticker, e);
                    }
                    Err(e) => {
                        eprintln!("Task join error: {}", e);
                    }
                }
            }

            Ok(())
        }
        Commands::Daily(args) => {
            let mut tasks = JoinSet::new();

            for ticker in &args.symbol {
                let ticker = ticker.clone();
                let av = Arc::clone(&av);
                tasks.spawn(async move {
                    debug!("Getting quote for symbol: {ticker}");
                    (ticker.clone(), av.daily(&ticker, args.full).await)
                });
            }

            while let Some(result) = tasks.join_next().await {
                match result {
                    Ok((_ticker, Ok(quote))) => {
                        writeln!(out, "{quote}")?;
                    }
                    Ok((ticker, Err(e))) => {
                        eprintln!("Error getting quote for {}: {}", ticker, e);
                    }
                    Err(e) => {
                        eprintln!("Task join error: {}", e);
                    }
                }
            }

            Ok(())
        }
    }
}
