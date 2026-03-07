use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;

pub struct PplxClient {
    client: Client,
    api_key: String,
}

impl PplxClient {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("PERPLEXITY_API_KEY")
            .context("PERPLEXITY_API_KEY not set. Add it to ~/.secrets")?;
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client, api_key })
    }

    pub fn query(&self, model: &str, query: &str) -> Result<Value> {
        let body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "Answer accurately based on what you can find. If you cannot confirm specific facts (e.g. whether a business exists at a location), say explicitly that you could not confirm it — do not assert absence. Distinguish between 'not found in results' and 'does not exist'."
                },
                {"role": "user", "content": query}
            ]
        });

        let resp = self
            .client
            .post("https://api.perplexity.ai/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .context("Failed to reach Perplexity API")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            bail!("Perplexity API returned {status}: {body}");
        }

        resp.json().context("Failed to parse API response")
    }
}
