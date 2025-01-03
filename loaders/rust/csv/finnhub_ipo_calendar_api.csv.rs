//! **Finnhub IPO Calendar API Tool**
//! 
//! Downloads upcoming IPO data with a 3-month forward window
//! Documentation: https://finnhub.io/docs/api/ipo-calendar
//!
//! **Usage**: rust-script finnhub_ipo_calendar_api.csv.rs > output.csv
//!
//! **Environment Setup**:
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.11", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! csv = "1.2"
//! chrono = { version = "0.4", features = ["serde"] }
//! ```

use serde::{Deserialize};
use std::error::Error;
use std::time::Instant;
use chrono::Local;
use serde_json::Value;
use std::collections::BTreeMap;

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
   let mut wtr = csv::Writer::from_writer(std::io::stdout());
   
   if let Some(first_item) = api_response.ipo_calendar.first() {
       let headers: Vec<String> = first_item.as_object()
           .unwrap_or(&serde_json::Map::new())
           .keys()
           .map(|k| clean_column_name(k))
           .collect();
       wtr.write_record(&headers)?;
   }

   for item in &api_response.ipo_calendar {
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
       }
   }

   wtr.flush()?;
   let processing_duration = processing_start.elapsed();
   
   eprintln!("\nTiming Information:");
   eprintln!("Data fetching: {:?}", data_duration);
   eprintln!("CSV processing: {:?}", processing_duration);
   eprintln!("Total execution time: {:?}", start.elapsed());
   eprintln!("\nData Information:");
   eprintln!("Date range: {} to {}", from_date, to_date);
   eprintln!("Total IPOs found: {}", api_response.ipo_calendar.len());

   Ok(())
}