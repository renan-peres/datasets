//! **AlphaVantage News Sentiment API Tool**
//! 
//! Downloads news articles and sentiment data from the last 2 days across multiple categories
//! Documentation: https://www.alphavantage.co/documentation/#news-sentiment
//!
//! **Usage**: rust-script alphavantage_news_api.csv.rs > output.csv
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
use chrono::prelude::*;
use serde_json::Value;
use std::collections::BTreeMap;

#[path = "../make_clean_names.rs"]
mod make_clean_names;
use make_clean_names::clean_column_name;


mod make_clean_names;

#[derive(Debug, Deserialize)]
struct AlphaVantageResponse {
    #[serde(default)]
    feed: Vec<Value>,
    #[serde(default)]
    items: Vec<Value>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let client = reqwest::Client::new();
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    let mut headers_written = false;
    let mut total_articles = 0;
    
    // Calculate time_from (2 days ago)
    let two_days_ago = Utc::now() - chrono::Duration::days(2);
    let time_from = two_days_ago.format("%Y%m%dT%H%M").to_string();
    
    let api_key = "WC1568Y6FXY98WA9";
    let categories = vec!["mergers_and_acquisitions", "earnings", "financial_markets", "finance", 
                        "real_estate", "ipo", "economy_monetary", "economy_fiscal", "economy_macro",
                        "blockchain", "energy_transportation", "retail_wholesale", "technology"];
                        
    for category in &categories {
        let data_start = Instant::now();
        let url = format!(
            "https://www.alphavantage.co/query?function=NEWS_SENTIMENT&topics={}&time_from={}&apikey={}", 
            category,
            time_from,
            api_key
        );
        
        eprintln!("Fetching news for category: {} from {}", category, time_from);
        
        let response = client
            .get(&url)
            .send()
            .await?;

        let news_data: Value = response.json().await?;
        let data_duration = data_start.elapsed();
        
        let processing_start = Instant::now();

        // Check if we have feed data
        if let Some(feed) = news_data.get("feed").and_then(|f| f.as_array()) {
            // Write headers only once
            if !headers_written {
                wtr.write_record(&[
                    "category",
                    "title",
                    "source",
                    "summary",
                    "url",
                    "time_published",
                    "sentiment_score",
                    "sentiment_label"
                ])?;
                headers_written = true;
            }

            // Write data from current category
            for article in feed {
                let record = vec![
                    category.to_string(),
                    article.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    article.get("source").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    article.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    article.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    article.get("time_published").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    article.get("overall_sentiment_score").and_then(|v| v.as_f64()).map_or("".to_string(), |f| f.to_string()),
                    article.get("overall_sentiment_label").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                ];
                wtr.write_record(&record)?;
                total_articles += 1;
            }
        }

        let processing_duration = processing_start.elapsed();
        
        eprintln!("\nCategory {} Information:", category);
        eprintln!("Data fetching: {:?}", data_duration);
        eprintln!("CSV processing: {:?}", processing_duration);
        eprintln!("Running time: {:?}", start.elapsed());
    }

    wtr.flush()?;
    
    eprintln!("\nFinal Summary:");
    eprintln!("Total execution time: {:?}", start.elapsed());
    eprintln!("Total articles processed: {}", total_articles);
    eprintln!("Time range: from {} to now", time_from);

    Ok(())
}