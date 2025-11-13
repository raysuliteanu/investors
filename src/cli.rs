use clap::{Args, Parser, Subcommand};
use log::debug;
use polars::prelude::*;
use std::{
    error::Error,
    fs::File,
    io::{self, Write},
    sync::Arc,
};
use tokio::task::JoinSet;

use crate::av::AV;

#[derive(Debug, Parser)]
pub struct Cli {
    /// specify API key; if missing, taken from the environment variable ALPHAVANTAGE_API_KEY
    #[arg(short, long)]
    pub key: Option<String>,
    /// output in csv (default is json)
    #[arg(long)]
    pub csv: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// get a stock quote
    Quote(QuoteArgs),
    /// get historical daily stock quote data
    Daily(DailyQuoteArgs),
    /// Read (load) a CSV file and write it out as a Parquet file
    Load(LoadArgs),
}

#[derive(Debug, Args, Default)]
pub struct LoadArgs {
    /// input csv file name
    #[arg(short, long)]
    input: String,
    /// output parquet file name
    #[arg(short, long)]
    output: String,
}

#[derive(Debug, Args, Default)]
pub struct QuoteArgs {
    /// write the output to a file instead of stdout
    #[arg(short, long)]
    output: Option<String>,
    /// stock ticker symbol
    #[arg(required = true)]
    symbol: Vec<String>,
}

#[derive(Debug, Args, Default)]
pub struct DailyQuoteArgs {
    /// write the output to a file instead of stdout
    #[arg(short, long)]
    output: Option<String>,
    /// if set, retrieve all data points; default is compact (latest 100 data points)
    #[arg(long)]
    full: bool,
    /// stock ticker symbol
    #[arg(required = true)]
    symbol: Vec<String>,
}

pub async fn do_load(args: LoadArgs) -> Result<(), Box<dyn Error>> {
    tokio::task::spawn_blocking(move || -> Result<(), PolarsError> {
        let input = PlPath::from_str(&args.input);
        let output = PlPath::from_str(&args.output);
        let target = SinkTarget::Path(output);
        let sink_options = SinkOptions::default();
        let write_options = ParquetWriteOptions::default();
        let csv = LazyCsvReader::new(input).with_has_header(true).finish()?;
        let lf = csv.sink_parquet(target, write_options, None, sink_options)?;
        let _ = lf.collect()?;
        Ok(())
    })
    .await
    .map_err(|e| Box::new(e) as Box<dyn Error>)??;
    Ok(())
}

pub async fn do_quote(args: QuoteArgs, key: String, csv: bool) -> Result<(), Box<dyn Error>> {
    let mut out = get_file(args.output)?;
    let av = Arc::new(AV::new(key, csv));
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

pub async fn do_daily_quote(
    args: DailyQuoteArgs,
    key: String,
    csv: bool,
) -> Result<(), Box<dyn Error>> {
    let mut out = get_file(args.output)?;
    let av = Arc::new(AV::new(key, csv));
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

fn get_file(name: Option<String>) -> Result<Box<dyn Write>, io::Error> {
    let file: Box<dyn Write> = if let Some(output) = name {
        Box::new(File::create(output)?)
    } else {
        Box::new(io::stdout().lock())
    };

    Ok(file)
}
