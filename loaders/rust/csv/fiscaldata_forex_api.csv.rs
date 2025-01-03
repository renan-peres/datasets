//! **Treasury Exchange Rates API Download Tool**
//!
//! **Usage**: rust-script fiscaldata_forex_api.csv.rs > output.csv
//! 
//! Downloads historical exchange rates data from the Treasury's FiscalData API using pagination
//! Documentation: https://fiscaldata.treasury.gov/api-documentation/
//!
//! **Environment Setup**:
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
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Instant;

#[path = "../make_clean_names.rs"]
mod make_clean_names;
use make_clean_names::clean_column_name;

#[derive(Debug, Deserialize)]
struct ApiResponse {
   data: Vec<Value>,
   meta: Meta,
}

#[derive(Debug, Deserialize)]
struct Meta {
   count: i32,
   #[allow(dead_code)]
   labels: Value,
   #[allow(dead_code)]
   #[serde(rename = "dataTypes")]
   data_types: Value,
   #[allow(dead_code)]
   #[serde(rename = "dataFormats")]
   data_formats: Value,
   #[serde(rename = "total-count")]
   total_count: i32,
   #[serde(rename = "total-pages")]
   total_pages: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   let start = Instant::now();
   let client = reqwest::Client::new();
   let mut wtr = csv::Writer::from_writer(std::io::stdout());
   let mut headers_written = false;
   let mut total_rows_processed = 0;
   
   let page_size = 10000;
   let mut current_page = 1;
   
   loop {
       let data_start = Instant::now();
       let response = client
           .get("https://api.fiscaldata.treasury.gov/services/api/fiscal_service/v1/accounting/od/rates_of_exchange")
           .query(&[
               ("filter", "record_date:gte:2020-01-01"),
               ("sort", "-record_date"),
               ("page[size]", &page_size.to_string()),
               ("page[number]", &current_page.to_string())
           ])
           .send()
           .await?;

       let api_response: ApiResponse = response.json().await?;
       let data_duration = data_start.elapsed();
       
       let processing_start = Instant::now();

       if !headers_written {
           if let Some(first_item) = api_response.data.first() {
               let headers: Vec<String> = first_item.as_object()
                   .unwrap_or(&serde_json::Map::new())
                   .keys()
                   .map(|k| clean_column_name(k))
                   .collect();
               wtr.write_record(&headers)?;
               headers_written = true;
           }
       }

       for item in api_response.data {
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
       
       let processing_duration = processing_start.elapsed();
       total_rows_processed += api_response.meta.count;

       eprintln!("\nPage {} Information:", current_page);
       eprintln!("Timing Information:");
       eprintln!("Data fetching: {:?}", data_duration);
       eprintln!("CSV processing: {:?}", processing_duration);
       eprintln!("Running time: {:?}", start.elapsed());
       eprintln!("\nData Information:");
       eprintln!("Rows in current response: {}", api_response.meta.count);
       eprintln!("Total available records: {}", api_response.meta.total_count);
       eprintln!("Total pages: {}", api_response.meta.total_pages);
       eprintln!("Processed {} of {} records", total_rows_processed, api_response.meta.total_count);

       if current_page >= api_response.meta.total_pages {
           break;
       }
       
       current_page += 1;
   }

   wtr.flush()?;
   
   eprintln!("\nFinal Summary:");
   eprintln!("Total execution time: {:?}", start.elapsed());
   eprintln!("Total records processed: {}", total_rows_processed);

   Ok(())
}