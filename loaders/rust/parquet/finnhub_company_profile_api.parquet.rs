//! **Finnhub Company Profiles API Tool**
//!
//! Fetches company profiles for US stocks using concurrent requests
//! Documentation: https://finnhub.io/docs/api/company-profile2
//!
//! **Usage**: rust-script finnhub_company_profile_api.parquet.rs > output.parquet
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

        sleep(Duration::from_millis(500)).await;
    }

    let processing_start = Instant::now();

    // Get schema from the first successful response
    let first_valid_response = all_results.iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|r| r.as_ref())
        .next();

    if let Some(first_item) = first_valid_response {
        if let Some(obj) = first_item.as_object() {
            // Create a mapping of clean names to original keys
            let field_mapping: Vec<(String, String)> = obj.keys()
                .map(|k| (clean_column_name(k), k.to_string()))
                .collect();

            // Create schema fields using clean names
            let schema_fields: Vec<Field> = field_mapping.iter().map(|(clean_name, original_key)| {
                let data_type = match obj.get(original_key).unwrap() {
                    Value::String(_) => DataType::Utf8,
                    Value::Number(_) => DataType::Float64,  // Treat all numbers as Float64
                    _ => DataType::Utf8,  // Default everything else to strings
                };
                Field::new(clean_name, data_type, true)
            }).collect();

            let schema = Schema::new(schema_fields);
            let mut writer = ArrowWriter::try_new(
                std::io::stdout(),
                Arc::new(schema.clone()),
                Some(WriterProperties::builder().build())
            )?;

            // Process all results
            let mut successful_entries = 0;

            // Create arrays for each field
            let mut arrays: Vec<Vec<Value>> = vec![Vec::new(); field_mapping.len()];

            // Collect all values using original keys
            for result in all_results {
                if let Ok(Some(item)) = result {
                    if let Some(obj) = item.as_object() {
                        for (i, (_, original_key)) in field_mapping.iter().enumerate() {
                            arrays[i].push(obj.get(original_key)
                                .unwrap_or(&Value::Null)
                                .clone());
                        }
                        successful_entries += 1;
                    }
                }
            }

            // Convert to Arrow arrays
            let arrow_arrays: Vec<ArrayRef> = field_mapping.iter().enumerate().map(|(i, _)| {
                let values = &arrays[i];
                let field = schema.field(i);
                match field.data_type() {
                    DataType::Utf8 => {
                        let string_values: Vec<String> = values.iter()
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                Value::Null => String::new(),
                                _ => v.to_string(),
                            })
                            .collect();
                        Arc::new(StringArray::from(string_values)) as ArrayRef
                    },
                    DataType::Float64 => {
                        let float_values: Vec<Option<f64>> = values.iter()
                            .map(|v| v.as_f64())
                            .collect();
                        Arc::new(Float64Array::from(float_values)) as ArrayRef
                    },
                    _ => {
                        // Default to string for any other types
                        let string_values: Vec<String> = values.iter()
                            .map(|v| v.to_string())
                            .collect();
                        Arc::new(StringArray::from(string_values)) as ArrayRef
                    }
                }
            }).collect();

            // Create and write record batch
            let batch = RecordBatch::try_new(
                Arc::new(schema),
                arrow_arrays,
            )?;

            writer.write(&batch)?;
            writer.close()?;

            let data_duration = data_start.elapsed();
            let processing_duration = processing_start.elapsed();
            
            eprintln!("\nTiming Information:");
            eprintln!("Data fetching: {:?}", data_duration);
            eprintln!("Parquet processing: {:?}", processing_duration);
            eprintln!("Total execution time: {:?}", start.elapsed());
            eprintln!("\nData Information:");
            eprintln!("Successful entries: {}/{}", successful_entries, symbols.len());
        }
    } else {
        eprintln!("No valid responses received to create schema");
    }

    Ok(())
}