use anyhow::{Context, Error};

use reqwest::Client;

pub struct AV {
    client: Client,
    api_url: String,
}

impl AV {
    const BASE_URL: &'static str = "https://www.alphavantage.co/query?";

    pub fn new(api_key: String) -> Self {
        let client = Client::new();
        AV {
            client,
            api_url: format!("{}apikey={}&", Self::BASE_URL, api_key),
        }
    }

    pub async fn quote(&self, symbol: &str) -> Result<String, Error> {
        let params = [("function", "GLOBAL_QUOTE"), ("symbol", symbol)];
        self.client
            .get(self.api_url.as_str())
            .query(&params)
            .send()
            .await?
            .text()
            .await
            .with_context(|| format!("Failed to get quote data for {}", symbol))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quote() {
        let key = std::env::var("ALPHAVANTAGE_API_KEY").expect("ALPHAVANTAGE_API_KEY not set");
        let av = AV::new(key);
        let result = av.quote("IBM").await;
        assert!(result.is_ok());
        println!("Quote data: {}", result.unwrap());
    }
}
