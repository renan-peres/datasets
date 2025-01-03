//! **Finnhub IPO Calendar API Tool**
//! 
//! Downloads upcoming IPO data with a 3-month forward window
//! Documentation: https://finnhub.io/docs/api/ipo-calendar
//!
//! **Usage**: rust-script finnhub_ipo_calendar_api.parquet.rs > output.parquet
//!
//! **Environment Setup**:
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! chrono = { version = "0.4", features = ["serde"] }
//! arrow = "50.0.0"
//! parquet = "50.0.0"
//! ```

use serde::{Deserialize};
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use chrono::Local;
use serde_json::Value;
use arrow::datatypes::{Schema, Field, DataType};
use arrow::array::{StringArray, Float64Array, ArrayRef};
use arrow::record_batch::RecordBatch;
use parquet::file::properties::WriterProperties;
use parquet::arrow::ArrowWriter;

#[path = "../make_clean_names.rs"]
mod make_clean_names;
use make_clean_names::clean_column_name;

#[derive(Debug, Deserialize)]
struct FinnhubResponse {
   #[serde(rename = "ipoCalendar")]
   ipo_calendar: Vec<Value>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   let start = Instant::now();
   let api_key = "ctl0tnpr01qn6d7jqpj0ctl0tnpr01qn6d7jqpjg";
   
   let now = Local::now();
   let from_date = now.format("%Y-%m-%d").to_string();
   let to_date = now.checked_add_months(chrono::Months::new(3))
       .unwrap()
       .format("%Y-%m-%d")
       .to_string();
   
   let data_start = Instant::now();
   let client = reqwest::Client::new();
   let response = client
       .get("https://finnhub.io/api/v1/calendar/ipo")
       .query(&[
           ("token", api_key),
           ("from", &from_date),
           ("to", &to_date)
       ])
       .send()
       .await?;

   let api_response: FinnhubResponse = response.json().await?;
   let data_duration = data_start.elapsed();
   
   let processing_start = Instant::now();

   if let Some(first_item) = api_response.ipo_calendar.first() {
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
           for item in &api_response.ipo_calendar {
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
   eprintln!("Data fetching: {:?}", data_duration);
   eprintln!("Parquet processing: {:?}", processing_duration);
   eprintln!("Total execution time: {:?}", start.elapsed());
   eprintln!("\nData Information:");
   eprintln!("Date range: {} to {}", from_date, to_date);
   eprintln!("Total IPOs found: {}", api_response.ipo_calendar.len());

   Ok(())
}