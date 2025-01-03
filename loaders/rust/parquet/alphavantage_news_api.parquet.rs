//! **AlphaVantage News Sentiment API Tool**
//! 
//! Downloads news articles and sentiment data from the last 2 days across multiple categories
//! Documentation: https://www.alphavantage.co/documentation/#news-sentiment
//!
//! **Usage**: rust-script alphavantage_news_api.parquet.rs > output.parquet
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
use chrono::prelude::*;
use serde_json::Value;
use arrow::datatypes::{Schema, Field, DataType};
use arrow::array::{StringArray, Float64Array, TimestampMillisecondArray, ArrayRef};
use arrow::record_batch::RecordBatch;
use parquet::file::properties::WriterProperties;
use parquet::arrow::ArrowWriter;

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
    let mut total_articles = 0;
    
    // Calculate time_from (2 days ago)
    let two_days_ago = Utc::now() - chrono::Duration::days(2);
    let time_from = two_days_ago.format("%Y%m%dT%H%M").to_string();
    
    // Define schema for Parquet output
    let schema = Schema::new(vec![
        Field::new("category", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, true),
        Field::new("source", DataType::Utf8, true),
        Field::new("summary", DataType::Utf8, true),
        Field::new("url", DataType::Utf8, true),
        Field::new("time_published", DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None), true),
        Field::new("sentiment_score", DataType::Float64, true),
        Field::new("sentiment_label", DataType::Utf8, true),
    ]);

    let mut writer = ArrowWriter::try_new(
        std::io::stdout(),
        Arc::new(schema.clone()),
        Some(WriterProperties::builder().build())
    )?;

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
            let mut categories = Vec::new();
            let mut titles = Vec::new();
            let mut sources = Vec::new();
            let mut summaries = Vec::new();
            let mut urls = Vec::new();
            let mut timestamps = Vec::new();
            let mut sentiment_scores = Vec::new();
            let mut sentiment_labels = Vec::new();

            for article in feed {
                categories.push(category.to_string());
                titles.push(article.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string());
                sources.push(article.get("source").and_then(|v| v.as_str()).unwrap_or_default().to_string());
                summaries.push(article.get("summary").and_then(|v| v.as_str()).unwrap_or_default().to_string());
                urls.push(article.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string());
                
                // Debug print the time_published format
                if let Some(ts_str) = article.get("time_published").and_then(|v| v.as_str()) {
                    eprintln!("Time published format: {}", ts_str);
                }
                
                // Convert time_published to timestamp
                let timestamp = article.get("time_published")
                .and_then(|v| v.as_str())
                .and_then(|ts| {
                    // First try the exact format YYYYMMDDTHHmmss
                    DateTime::parse_from_str(ts, "%Y%m%dT%H%M%S")
                        .or_else(|_| {
                            // If that fails, try parsing with just minutes (no seconds)
                            // by padding with "00" seconds
                            DateTime::parse_from_str(&format!("{}00", ts), "%Y%m%dT%H%M%S")
                        })
                        .map(|dt| dt.timestamp_millis())
                        .ok()
                })
                .unwrap_or_else(|| {
                    eprintln!("Failed to parse timestamp: {:?}", 
                        article.get("time_published").and_then(|v| v.as_str()));
                    0
                });
                timestamps.push(timestamp);
                
                sentiment_scores.push(article.get("overall_sentiment_score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0));
                sentiment_labels.push(article.get("overall_sentiment_label")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string());
                
                total_articles += 1;
            }

            // Create Arrow arrays
            let arrays: Vec<ArrayRef> = vec![
                Arc::new(StringArray::from(categories)),
                Arc::new(StringArray::from(titles)),
                Arc::new(StringArray::from(sources)),
                Arc::new(StringArray::from(summaries)),
                Arc::new(StringArray::from(urls)),
                Arc::new(TimestampMillisecondArray::from(timestamps)),
                Arc::new(Float64Array::from(sentiment_scores)),
                Arc::new(StringArray::from(sentiment_labels)),
            ];

            // Create and write record batch
            let batch = RecordBatch::try_new(Arc::new(schema.clone()), arrays)?;
            writer.write(&batch)?;
        }

        let processing_duration = processing_start.elapsed();
        
        eprintln!("\nCategory {} Information:", category);
        eprintln!("Data fetching: {:?}", data_duration);
        eprintln!("Parquet processing: {:?}", processing_duration);
        eprintln!("Running time: {:?}", start.elapsed());
    }

    // Close the writer
    writer.close()?;
    
    eprintln!("\nFinal Summary:");
    eprintln!("Total execution time: {:?}", start.elapsed());
    eprintln!("Total articles processed: {}", total_articles);
    eprintln!("Time range: from {} to now", time_from);

    Ok(())
}