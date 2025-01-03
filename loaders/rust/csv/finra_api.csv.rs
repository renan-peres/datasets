//! **FINRA Treasury Monthly Aggregates API Tool**
//!
//! **Usage**:rust-script finra_api.csv.rs > output.csv
//!
//! Downloads treasury monthly aggregates data from FINRA's API using OAuth2 authentication
//! Documentation: https://developer.finra.org/docs#operation/getTreasuryMonthlyAggregates
//!
//! **Features**:
//! - OAuth2 authentication with FINRA API
//! - Automatic token refresh
//! - Clean CSV output with normalized column names
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
//! csv = "1.2"
//! ```

use serde::{Deserialize};
use std::error::Error;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::Instant;

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
   let mut wtr = csv::Writer::from_writer(std::io::stdout());
   
   if let Some(first_item) = market_data.first() {
       let headers: Vec<String> = first_item.as_object()
           .unwrap_or(&serde_json::Map::new())
           .keys()
           .map(|k| clean_column_name(k))
           .collect();
       wtr.write_record(&headers)?;
   }

   for item in market_data {
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
   eprintln!("Token acquisition: {:?}", token_duration);
   eprintln!("Data fetching: {:?}", data_duration);
   eprintln!("CSV processing: {:?}", processing_duration);
   eprintln!("Total execution time: {:?}", start.elapsed());
   eprintln!("\nData Information:");
   eprintln!("Total rows processed: {}", row_count);

   Ok(())
}