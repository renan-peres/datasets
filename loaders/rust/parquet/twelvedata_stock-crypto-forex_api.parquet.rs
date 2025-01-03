//! This script fetches cryptocurrency time-series data from the Twelve Data API.
//!
//! **Usage**: rust-script twelvedata_stock-crypto-forex_api.parquet.rs > output.parquet
//! 
//! Documentation: https://twelvedata.com/docs#time-series
//!
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! arrow = "50.0.0"
//! parquet = "50.0.0"
//! chrono = { version = "0.4", features = ["serde"] }
//! ```

use serde::{Deserialize};
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use serde_json::Value;
use arrow::datatypes::{Schema, Field, DataType, TimeUnit};
use arrow::array::{StringArray, Float64Array, ArrayRef, TimestampMillisecondArray};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use chrono::{NaiveDateTime};

#[path = "../make_clean_names.rs"]
mod make_clean_names;
use make_clean_names::clean_column_name;

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    code: i32,
    message: String
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ApiResponse {
    Success(Value),
    Error(ErrorResponse)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let api_key = "cffcf251cc0243f198823fa3e34ddf65";
    let symbols = vec!["BTC/USD", "ETH/USD", "SOL/USD", "XRP/USD"];
    let client = reqwest::Client::new();
    
    let mut all_arrays: Option<Vec<Vec<Value>>> = None;
    let mut schema: Option<Schema> = None;
    let mut writer: Option<ArrowWriter<std::io::Stdout>> = None;

    for symbol in symbols {
        let response = client
            .get("https://api.twelvedata.com/time_series")
            .query(&[
                ("symbol", symbol),
                ("interval", "1min"),
                ("apikey", api_key),
                ("outputsize", "100")
            ])
            .send()
            .await?;

        let api_response: ApiResponse = response.json().await?;

        match api_response {
            ApiResponse::Success(data) => {
                if let Some(values) = data.get("values").and_then(|v| v.as_array()) {
                    if schema.is_none() {
                        if let Some(first_item) = values.first() {
                            if let Some(obj) = first_item.as_object() {
                                let mut field_mapping = vec![(String::from("symbol"), String::from("symbol"))];
                                field_mapping.extend(obj.keys().map(|k| {
                                    (clean_column_name(k), k.to_string())
                                }));

                                // Create schema fields
                                let schema_fields: Vec<Field> = field_mapping.iter().map(|(clean_name, _)| {
                                    if clean_name == "symbol" {
                                        Field::new(clean_name, DataType::Utf8, false)
                                    } else if clean_name.contains("datetime") {
                                        Field::new(clean_name, DataType::Timestamp(TimeUnit::Millisecond, None), true)
                                    } else {
                                        Field::new(clean_name, DataType::Float64, true)
                                    }
                                }).collect();

                                schema = Some(Schema::new(schema_fields));
                                writer = Some(ArrowWriter::try_new(
                                    std::io::stdout(),
                                    Arc::new(schema.as_ref().unwrap().clone()),
                                    Some(WriterProperties::builder().build())
                                )?);
                                
                                all_arrays = Some(vec![Vec::new(); field_mapping.len()]);
                            }
                        }
                    }

                    if let (Some(arrays), Some(schema_ref)) = (all_arrays.as_mut(), &schema) {
                        for value in values {
                            if let Some(obj) = value.as_object() {
                                arrays[0].push(Value::String(symbol.to_string()));
                                
                                for (i, field) in schema_ref.fields().iter().enumerate().skip(1) {
                                    let value = obj.get(field.name())
                                        .unwrap_or(&Value::Null)
                                        .clone();
                                    arrays[i].push(value);
                                }
                            }
                        }
                    }
                }
            },
            ApiResponse::Error(err) => {
                eprintln!("API Error for {}: {} (Code: {})", symbol, err.message, err.code);
                continue;
            }
        }
    }

    if let (Some(arrays), Some(schema_ref), Some(mut writer)) = (all_arrays, schema, writer) {
        let arrow_arrays: Vec<ArrayRef> = schema_ref.fields().iter().enumerate().map(|(i, field)| {
            let values = &arrays[i];
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
                DataType::Timestamp(TimeUnit::Millisecond, _) => {
                    let timestamps: Vec<Option<i64>> = values.iter()
                        .map(|v| match v {
                            Value::String(s) => {
                                NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                                    .map(|dt| dt.and_utc().timestamp_millis())
                                    .ok()
                            },
                            _ => None,
                        })
                        .collect();
                    Arc::new(TimestampMillisecondArray::from(timestamps)) as ArrayRef
                },
                DataType::Float64 => {
                    let float_values: Vec<Option<f64>> = values.iter()
                        .map(|v| match v {
                            Value::String(s) => s.parse::<f64>().ok(),
                            Value::Number(n) => n.as_f64(),
                            _ => None,
                        })
                        .collect();
                    Arc::new(Float64Array::from(float_values)) as ArrayRef
                },
                _ => {
                    let string_values: Vec<String> = values.iter()
                        .map(|v| v.to_string())
                        .collect();
                    Arc::new(StringArray::from(string_values)) as ArrayRef
                }
            }
        }).collect();

        let batch = RecordBatch::try_new(Arc::new(schema_ref), arrow_arrays)?;
        writer.write(&batch)?;
        writer.close()?;
    }

    eprintln!("Total execution time: {:?}", start.elapsed());

    Ok(())
}