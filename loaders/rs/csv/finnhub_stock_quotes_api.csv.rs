//! **Finnhub Stock Market Data API Tool**
//!
//! Fetches real-time quotes and company profiles for US stocks using concurrent requests
//! Documentation: https://finnhub.io/docs/api/quote
//!
//! **Usage**: rust-script finnhub_stock_quotes_api.csv.rs > output.csv
//!
//! **Features**:
//! - Concurrent API requests for better performance
//! - Real-time quotes and company profiles
//! - Major US stocks across sectors
//! - Error handling and logging
//! - Clean CSV output format with proper data types
//!
//! **Environment Setup**:
//! ```cargo
//! [package]
//! edition = "2021"
//! 
//! [dependencies]
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! csv = "1.2"
//! futures = "0.3"
//! ```

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::error::Error;
use std::time::{Instant, Duration};
use futures::future;
use tokio::time::sleep;

#[path = "../finnhub_config.rs"]
mod config;

#[derive(Debug, Serialize, Deserialize)]
struct StockQuote {
    symbol: String,
    current_price: f64,
    change: f64,
    percent_change: f64,
    high_price: f64,
    low_price: f64,
    open_price: f64,
    previous_close: f64,
}

async fn fetch_quote(
    client: &reqwest::Client,
    symbol: &str,
    api_key: &str,
    retry_count: u32,
) -> Result<Option<Value>, Box<dyn Error + Send + Sync>> {
    for attempt in 0..retry_count {
        if attempt > 0 {
            sleep(Duration::from_millis(100 * (attempt as u64))).await;
        }

        let quote_response = client
            .get("https://finnhub.io/api/v1/quote")
            .query(&[
                ("token", api_key),
                ("symbol", &symbol)
            ])
            .send()
            .await?;

        // Check if we hit the rate limit
        if quote_response.status() == 429 {
            eprintln!("Rate limit hit for {}, waiting before retry...", symbol);
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        let response_text = quote_response.text().await?;
        if response_text.trim().is_empty() {
            eprintln!("Empty response for symbol: {}", symbol);
            return Ok(None);
        }

        match serde_json::from_str::<Value>(&response_text) {
            Ok(quote) => {
                // Verify that we have all required fields
                if let Some(obj) = quote.as_object() {
                    if obj.contains_key("c") && obj.contains_key("d") && 
                       obj.contains_key("dp") && obj.contains_key("h") && 
                       obj.contains_key("l") && obj.contains_key("o") && 
                       obj.contains_key("pc") {
                        eprintln!("Successfully processed quote for: {}", symbol);
                        return Ok(Some(quote));
                    }
                }
                eprintln!("Incomplete quote data for symbol: {}", symbol);
                if attempt == retry_count - 1 {
                    return Ok(None);
                }
            },
            Err(e) => {
                eprintln!("Error parsing quote data for {}: {}", symbol, e);
                if attempt == retry_count - 1 {
                    return Ok(None);
                }
            }
        }
    }
    Ok(None)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let symbols = config::get_symbols();
    
    let data_start = Instant::now();
    let client = reqwest::Client::new();

    // Process symbols in smaller chunks to avoid rate limiting
    let chunk_size = 10;
    let mut all_results = Vec::new();

    for chunk in symbols.chunks(chunk_size) {
        let requests = chunk.iter().map(|symbol| {
            let client = &client;
            let api_key = config::FINNHUB_API_KEY;
            let symbol = symbol.to_string();
            
            async move {
                fetch_quote(client, &symbol, api_key, 3).await
            }
        });

        let chunk_results = future::join_all(requests).await;
        all_results.extend(chunk_results);

        // Add delay between chunks to avoid rate limiting
        sleep(Duration::from_millis(500)).await;
    }

    let processing_start = Instant::now();
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    
    // Write headers with type hints
    wtr.write_record(&[
        "symbol",
        "current_price",
        "change",
        "percent_change",
        "high_price",
        "low_price",
        "open_price",
        "previous_close"
    ])?;

    let mut successful_entries = 0;

    for (symbol, result) in symbols.iter().zip(all_results.iter()) {
        if let Ok(Some(item)) = result {
            if let Some(obj) = item.as_object() {
                let record = vec![
                    symbol.to_string(),
                    obj.get("c").and_then(|v| v.as_f64()).map(|v| v.to_string()).unwrap_or_default(),
                    obj.get("d").and_then(|v| v.as_f64()).map(|v| v.to_string()).unwrap_or_default(),
                    obj.get("dp").and_then(|v| v.as_f64()).map(|v| v.to_string()).unwrap_or_default(),
                    obj.get("h").and_then(|v| v.as_f64()).map(|v| v.to_string()).unwrap_or_default(),
                    obj.get("l").and_then(|v| v.as_f64()).map(|v| v.to_string()).unwrap_or_default(),
                    obj.get("o").and_then(|v| v.as_f64()).map(|v| v.to_string()).unwrap_or_default(),
                    obj.get("pc").and_then(|v| v.as_f64()).map(|v| v.to_string()).unwrap_or_default(),
                ];
                wtr.write_record(&record)?;
                successful_entries += 1;
            }
        }
    }

    wtr.flush()?;
    let data_duration = data_start.elapsed();
    let processing_duration = processing_start.elapsed();
    
    eprintln!("\nTiming Information:");
    eprintln!("Data fetching: {:?}", data_duration);
    eprintln!("CSV processing: {:?}", processing_duration);
    eprintln!("Total execution time: {:?}", start.elapsed());
    eprintln!("\nData Information:");
    eprintln!("Successful entries: {}/{}", successful_entries, symbols.len());

    Ok(())
}