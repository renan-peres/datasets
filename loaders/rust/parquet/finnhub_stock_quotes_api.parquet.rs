//! **Finnhub Stock Market Data API Tool**
//!
//! Fetches real-time quotes and company profiles for US stocks using concurrent requests
//! Documentation: https://finnhub.io/docs/api/quote
//!
//! **Usage**: rust-script finnhub_stock_quotes_api.parquet.rs > output.parquet
//!
//! **Environment Setup**:
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! futures = "0.3"
//! arrow = "50.0.0"
//! parquet = "50.0.0"
//! ```

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;
use std::time::{Instant, Duration};
use futures::future;
use tokio::time::sleep;
use arrow::datatypes::{Schema, Field, DataType};
use arrow::array::{StringArray, Float64Array, ArrayRef};
use arrow::record_batch::RecordBatch;
use parquet::file::properties::WriterProperties;
use parquet::arrow::ArrowWriter;

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

    // Define schema for Parquet output
    let schema = Schema::new(vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new("current_price", DataType::Float64, true),
        Field::new("change", DataType::Float64, true),
        Field::new("percent_change", DataType::Float64, true),
        Field::new("high_price", DataType::Float64, true),
        Field::new("low_price", DataType::Float64, true),
        Field::new("open_price", DataType::Float64, true),
        Field::new("previous_close", DataType::Float64, true),
    ]);

    let mut writer = ArrowWriter::try_new(
        std::io::stdout(),
        Arc::new(schema.clone()),
        Some(WriterProperties::builder().build())
    )?;

    // Process symbols in smaller chunks to avoid rate limiting
    let chunk_size = 10;
    let mut successful_entries = 0;

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
        
        // Process chunk results and write to Parquet
        let mut symbols = Vec::new();
        let mut current_prices = Vec::new();
        let mut changes = Vec::new();
        let mut percent_changes = Vec::new();
        let mut high_prices = Vec::new();
        let mut low_prices = Vec::new();
        let mut open_prices = Vec::new();
        let mut previous_closes = Vec::new();

        for (symbol, result) in chunk.iter().zip(chunk_results.iter()) {
            if let Ok(Some(item)) = result {
                if let Some(obj) = item.as_object() {
                    symbols.push(symbol.to_string());
                    current_prices.push(obj.get("c").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    changes.push(obj.get("d").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    percent_changes.push(obj.get("dp").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    high_prices.push(obj.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    low_prices.push(obj.get("l").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    open_prices.push(obj.get("o").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    previous_closes.push(obj.get("pc").and_then(|v| v.as_f64()).unwrap_or(0.0));
                    successful_entries += 1;
                }
            }
        }

        // Create Arrow arrays
        let arrays: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(symbols)),
            Arc::new(Float64Array::from(current_prices)),
            Arc::new(Float64Array::from(changes)),
            Arc::new(Float64Array::from(percent_changes)),
            Arc::new(Float64Array::from(high_prices)),
            Arc::new(Float64Array::from(low_prices)),
            Arc::new(Float64Array::from(open_prices)),
            Arc::new(Float64Array::from(previous_closes)),
        ];

        // Create and write record batch
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), arrays)?;
        writer.write(&batch)?;

        // Add delay between chunks to avoid rate limiting
        sleep(Duration::from_millis(500)).await;
    }

    // Close the writer
    writer.close()?;

    let data_duration = data_start.elapsed();
    
    eprintln!("\nTiming Information:");
    eprintln!("Data fetching and processing: {:?}", data_duration);
    eprintln!("Total execution time: {:?}", start.elapsed());
    eprintln!("\nData Information:");
    eprintln!("Successful entries: {}/{}", successful_entries, symbols.len());

    Ok(())
}