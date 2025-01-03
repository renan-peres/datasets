//! **FINRA Treasury Monthly Aggregates API Tool**
//!
//! **Usage**: rust-script finra_api.parquet.rs > output.parquet
//!
//! Downloads treasury monthly aggregates data from FINRA's API using OAuth2 authentication
//! Documentation: https://developer.finra.org/docs#operation/getTreasuryMonthlyAggregates
//!
//! **Features**:
//! - OAuth2 authentication with FINRA API
//! - Automatic token refresh
//! - Clean Parquet output with normalized column names
//! - Detailed timing and processing statistics
//!
//! **Environment Setup**:
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! base64 = "0.21"
//! arrow = "50.0.0"
//! parquet = "50.0.0"
//! ```

use serde::{Deserialize};
use std::error::Error;
use std::sync::Arc;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use std::time::Instant;
use arrow::datatypes::{Schema, Field, DataType};
use arrow::array::{StringArray, Float64Array, ArrayRef};
use arrow::record_batch::RecordBatch;
use parquet::file::properties::WriterProperties;
use parquet::arrow::ArrowWriter;

#[path = "../make_clean_names.rs"]
mod make_clean_names;
use make_clean_names::clean_column_name;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    
    let client = reqwest::Client::new();
    let auth_string = STANDARD.encode("445db239b8054bbb9542:kbj.uxc5TCQ-vah9cqr");
    
    let token_start = Instant::now();
    let token_response = client
        .post("https://ews.fip.finra.org/fip/rest/ews/oauth2/access_token?grant_type=client_credentials")
        .header("Authorization", format!("Basic {}", auth_string))
        .send()
        .await?;
    let token_text = token_response.text().await?;
    let token: TokenResponse = serde_json::from_str(&token_text)?;
    let token_duration = token_start.elapsed();
    
    let data_start = Instant::now();
    let market_response = client
        .get("https://api.finra.org/data/group/fixedIncomeMarket/name/treasuryMonthlyAggregates")
        .query(&[("limit", "10000")])
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Accept", "application/json")
        .send()
        .await?;

    let market_data: Vec<Value> = serde_json::from_str(&market_response.text().await?)?;
    let row_count = market_data.len();
    let data_duration = data_start.elapsed();
    
    let processing_start = Instant::now();

    if let Some(first_item) = market_data.first() {
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

            // Create arrays for each field
            let mut arrays: Vec<Vec<Value>> = vec![Vec::new(); field_mapping.len()];

            // Collect all values using original keys
            for item in &market_data {
                if let Some(obj) = item.as_object() {
                    for (i, (_, original_key)) in field_mapping.iter().enumerate() {
                        arrays[i].push(obj.get(original_key)
                            .unwrap_or(&Value::Null)
                            .clone());
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
            let batch = RecordBatch::try_new(Arc::new(schema), arrow_arrays)?;
            writer.write(&batch)?;
            writer.close()?;
        }
    }

    let processing_duration = processing_start.elapsed();
    
    eprintln!("\nTiming Information:");
    eprintln!("Token acquisition: {:?}", token_duration);
    eprintln!("Data fetching: {:?}", data_duration);
    eprintln!("Parquet processing: {:?}", processing_duration);
    eprintln!("Total execution time: {:?}", start.elapsed());
    eprintln!("\nData Information:");
    eprintln!("Total rows processed: {}", row_count);

    Ok(())
}