use anyhow::{Context, Error};

use reqwest::Client;

pub struct AV {
    client: Client,
    api_url: String,
    format: String,
}

impl AV {
    const BASE_URL: &'static str = "https://www.alphavantage.co/query?";

    pub fn new(api_key: String, csv: bool) -> Self {
        let client = Client::new();
        let format = if csv { "csv" } else { "json" };
        AV {
            client,
            api_url: format!("{}apikey={}&", Self::BASE_URL, api_key),
            format: format.to_string(),
        }
    }

    async fn execute(&self, params: &[(&str, &str)]) -> Result<String, reqwest::Error> {
        self.client
            .get(self.api_url.as_str())
            .query(&params)
            .send()
            .await?
            .text()
            .await
    }

    pub async fn daily(&self, symbol: &str, full: bool) -> Result<String, Error> {
        let size = if full { "full" } else { "compact" };
        let params = [
            ("function", "TIME_SERIES_DAILY"),
            ("symbol", symbol),
            ("outputsize", size),
            ("datatype", self.format.as_str()),
        ];
        self.execute(&params)
            .await
            .with_context(|| format!("Failed to get daily quote data for {}", symbol))
    }

    pub async fn quote(&self, symbol: &str) -> Result<String, Error> {
        let params = [
            ("function", "GLOBAL_QUOTE"),
            ("symbol", symbol),
            ("datatype", self.format.as_str()),
        ];
        self.execute(&params)
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
        let av = AV::new(key, false);
        let result = av.quote("IBM").await;
        assert!(result.is_ok());
    }
}
