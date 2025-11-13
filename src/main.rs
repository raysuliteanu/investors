use std::error::Error;

use clap::Parser;
use investors::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    // TODO: if key not on cmd line and not in env, then notify user and exit
    let key = args
        .key
        .unwrap_or_else(|| std::env::var("ALPHAVANTAGE_API_KEY").unwrap());

    let csv = args.csv;
    match args.command {
        Commands::Load(args) => {
            investors::cli::do_load(args).await?;
        }
        Commands::Quote(args) => {
            investors::cli::do_quote(args, key, csv).await?;
        }
        Commands::Daily(args) => {
            investors::cli::do_daily_quote(args, key, csv).await?;
        }
    }

    Ok(())
}
