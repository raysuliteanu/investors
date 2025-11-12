use clap::Parser;
use investors::av::AV;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    key: Option<String>,
    symbol: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    // TODO: if key not on cmd line and not in env, then notify user and exit
    let key = args
        .key
        .unwrap_or_else(|| std::env::var("ALPHAVANTAGE_API_KEY").unwrap());

    let av = AV::new(key);
    let quote = av.quote(&args.symbol).await?;
    println!("{quote}");
    Ok(())
}
