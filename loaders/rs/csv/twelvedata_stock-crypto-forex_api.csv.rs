//! This script fetches cryptocurrency time-series data from the Twelve Data API.
//!
//! **Usage**: rust-script twelvedata_stock-crypto-forex_api.csv.rs > output.csv
//! 
//! Documentation: https://twelvedata.com/docs#time-series
//! 
//! Usage:
//! 1. Add your Twelve Data API key
//! 2. Run script to fetch price data for specified crypto pairs
//! 3. Output is in CSV format with the following columns:
//!    symbol,datetime,open,high,low,close,volume
//!
//! Sample output:
//! symbol,close,datetime,high,low,open
//! BTC/USD,96063.1,2024-12-26 16:01:00,96071.03,95969.12,96043.02
//! ETH/USD,3340.07,2024-12-26 16:34:00,3343.14,3340.07,3342.73
//!
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! csv = "1.2"
//! ```

use serde::{Deserialize};
use std::error::Error;
use std::time::Instant;
use serde_json::Value;
use std::collections::BTreeMap;

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
   let mut wtr = csv::Writer::from_writer(std::io::stdout());
   let mut headers_written = false;

   let client = reqwest::Client::new();

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
                   if !headers_written {
                       if let Some(first_item) = values.first() {
                           let mut headers: Vec<String> = first_item.as_object()
                               .unwrap_or(&serde_json::Map::new())
                               .keys()
                               .map(|k| clean_column_name(k))
                               .collect();
                           headers.insert(0, "symbol".to_string());
                           wtr.write_record(&headers)?;
                           headers_written = true;
                       }
                   }

                   for value in values {
                       if let Some(obj) = value.as_object() {
                           let mut ordered: BTreeMap<_, _> = obj.iter()
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
                           ordered.insert("symbol".to_string(), symbol.to_string());
                           let record: Vec<String> = ordered.values().cloned().collect();
                           wtr.write_record(&record)?;
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

   wtr.flush()?;
   eprintln!("Total execution time: {:?}", start.elapsed());

   Ok(())
}