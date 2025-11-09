use anyhow::Error;
use reqwest::blocking::Client;

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

    pub fn quote(&self, symbol: &str) -> Result<String, anyhow::Error> {
        let params = [("function", "GLOBAL_QUOTE"), ("symbol", symbol)];
        // TODO: handle errors properly (custom error class and/or better anyhow usage)
        self.client
            .get(self.api_url.as_str())
            .query(&params)
            .send()?
            .text()
            .map_err(|e| Error::msg(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote() {
        let key = std::env::var("ALPHAVANTAGE_API_KEY").expect("API key not set");
        let av = AV::new(key);
        let result = av.quote("IBM");
        assert!(result.is_ok());
        println!("Quote data: {}", result.unwrap());
    }
}
