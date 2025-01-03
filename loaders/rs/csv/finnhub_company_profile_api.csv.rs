//! **Finnhub Company Profiles API Tool**
//!
//! Fetches company profiles for US stocks using concurrent requests
//! Documentation: https://finnhub.io/docs/api/company-profile2
//!
//! **Usage**: rust-script finnhub_company_profile_api.csv.rs > output.csv
//!
//! **Features**:
//! - Concurrent API requests for better performance
//! - Real-time quotes and company profiles
//! - Major US stocks across sectors
//! - Error handling and logging
//! - Clean CSV output format
//!
//! **Environment Setup**:
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! csv = "1.2"
//! futures = "0.3"
//! ```

use serde_json::Value;
use std::error::Error;
use std::time::{Instant, Duration};
use futures::future;
use std::collections::BTreeMap;
use tokio::time::sleep;

#[path = "../finnhub_config.rs"]
mod config;

#[path = "../make_clean_names.rs"]
mod make_clean_names;
use make_clean_names::clean_column_name;

async fn fetch_profile(
    client: &reqwest::Client,
    symbol: &str,
    api_key: &str,
    retry_count: u32,
) -> Result<Option<Value>, Box<dyn Error + Send + Sync>> {
    for attempt in 0..retry_count {
        if attempt > 0 {
            sleep(Duration::from_millis(100 * (attempt as u64))).await;
        }

        let profile_response = client
            .get("https://finnhub.io/api/v1/stock/profile2")
            .query(&[
                ("token", api_key),
                ("symbol", &symbol)
            ])
            .send()
            .await?;

        // Check if we hit the rate limit
        if profile_response.status() == 429 {
            eprintln!("Rate limit hit for {}, waiting before retry...", symbol);
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        let response_text = profile_response.text().await?;
        if response_text.trim().is_empty() {
            eprintln!("Empty response for symbol: {}", symbol);
            return Ok(None);
        }

        match serde_json::from_str::<Value>(&response_text) {
            Ok(profile) => {
                if profile.as_object().map_or(false, |obj| !obj.is_empty()) {
                    eprintln!("Successfully processed profile for: {}", symbol);
                    return Ok(Some(profile));
                } else {
                    eprintln!("Empty profile data for symbol: {}", symbol);
                    return Ok(None);
                }
            },
            Err(e) => {
                eprintln!("Error parsing profile data for {}: {}", symbol, e);
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
                fetch_profile(client, &symbol, api_key, 3).await
            }
        });

        let chunk_results = future::join_all(requests).await;
        all_results.extend(chunk_results);

        // Add delay between chunks to avoid rate limiting
        sleep(Duration::from_millis(500)).await;
    }

    let processing_start = Instant::now();
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    
    // Get headers from the first successful response
    let first_valid_response = all_results.iter()
        .filter_map(|r: &Result<Option<Value>, Box<dyn Error + Send + Sync>>| r.as_ref().ok())
        .filter_map(|r: &Option<Value>| r.as_ref())
        .next();

    if let Some(first_item) = first_valid_response {
        if let Some(obj) = first_item.as_object() {
            let headers: Vec<String> = obj.keys()
                .map(|k| clean_column_name(k))
                .collect();
            wtr.write_record(&headers)?;
        }
    }

    let mut successful_entries = 0;
    for result in all_results {
        if let Ok(Some(item)) = result {
            if let Some(obj) = item.as_object() {
                let ordered: BTreeMap<_, _> = obj.iter()
                    .map(|(k, v)| {
                        let value = match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => String::from(""),
                            _ => v.to_string(),
                        };
                        (clean_column_name(k), value)
                    })
                    .collect();
                
                let record: Vec<String> = ordered.values().cloned().collect();
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