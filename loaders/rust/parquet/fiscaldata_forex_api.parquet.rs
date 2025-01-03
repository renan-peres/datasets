//! **Treasury Exchange Rates API Download Tool**
//!
//! **Usage**: rust-script fiscaldata_forex_api.parquet.rs > output.parquet
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
//! arrow = "50.0.0"
//! parquet = "50.0.0"
//! ```

use serde::{Deserialize};
use std::{error::Error, sync::Arc};
use serde_json::Value;
use arrow::datatypes::{Schema, Field, DataType};
use arrow::array::{StringArray, Int64Array, UInt64Array, Float64Array, BooleanArray, ArrayRef};
use arrow::record_batch::RecordBatch;
use parquet::file::properties::WriterProperties;
use parquet::arrow::ArrowWriter;
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
    let mut total_rows_processed = 0;
    let mut schema = None;
    let mut writer = None;
    
    let page_size = 10000;
    let mut current_page = 1;
    
    // First request to get schema
    let response = client
        .get("https://api.fiscaldata.treasury.gov/services/api/fiscal_service/v1/accounting/od/rates_of_exchange")
        .query(&[
            ("filter", "record_date:gte:2020-01-01"),
            ("sort", "-record_date"),
            ("page[size]", "1"),
            ("page[number]", "1")
        ])
        .send()
        .await?;

    let initial_response: ApiResponse = response.json().await?;
    
    if let Some(first_item) = initial_response.data.first() {
        let obj = first_item.as_object().unwrap();
        let schema_fields: Vec<Field> = obj.iter().map(|(k, v)| {
            let data_type = match v {
                Value::String(_) => DataType::Utf8,
                Value::Number(n) => {
                    if n.is_i64() { DataType::Int64 }
                    else if n.is_u64() { DataType::UInt64 }
                    else { DataType::Float64 }
                },
                Value::Bool(_) => DataType::Boolean,
                _ => DataType::Utf8,
            };
            Field::new(&clean_column_name(k), data_type, true)
        }).collect();
        
        schema = Some(Schema::new(schema_fields));
        writer = Some(ArrowWriter::try_new(
            std::io::stdout(),
            Arc::new(schema.as_ref().unwrap().clone()),
            Some(WriterProperties::builder().build())
        )?);
    }
    
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

        if let (Some(writer_ref), Some(schema_ref)) = (writer.as_mut(), schema.as_ref()) {
            let mut columns: Vec<ArrayRef> = Vec::new();
            
            for field in schema_ref.fields() {
                let column_data = api_response.data.iter().map(|item| {
                    let obj = item.as_object().unwrap();
                    let value = &obj[field.name()];
                    value.clone()
                }).collect::<Vec<Value>>();

                let array: ArrayRef = match field.data_type() {
                    DataType::Utf8 => Arc::new(StringArray::from(
                        column_data.iter()
                            .map(|v| v.as_str().unwrap_or(""))
                            .collect::<Vec<&str>>()
                    )),
                    DataType::Int64 => Arc::new(Int64Array::from(
                        column_data.iter()
                            .map(|v| v.as_i64().unwrap_or(0))
                            .collect::<Vec<i64>>()
                    )),
                    DataType::UInt64 => Arc::new(UInt64Array::from(
                        column_data.iter()
                            .map(|v| v.as_u64().unwrap_or(0))
                            .collect::<Vec<u64>>()
                    )),
                    DataType::Float64 => Arc::new(Float64Array::from(
                        column_data.iter()
                            .map(|v| v.as_f64().unwrap_or(0.0))
                            .collect::<Vec<f64>>()
                    )),
                    DataType::Boolean => Arc::new(BooleanArray::from(
                        column_data.iter()
                            .map(|v| v.as_bool().unwrap_or(false))
                            .collect::<Vec<bool>>()
                    )),
                    _ => panic!("Unsupported data type"),
                };
                columns.push(array);
            }

            let batch = RecordBatch::try_new(
                Arc::new(schema_ref.clone()),
                columns,
            )?;
            
            writer_ref.write(&batch)?;
        }
        
        let processing_duration = processing_start.elapsed();
        total_rows_processed += api_response.meta.count;

        eprintln!("\nPage {} Information:", current_page);
        eprintln!("Data fetching: {:?}", data_duration);
        eprintln!("Parquet processing: {:?}", processing_duration);
        eprintln!("Running time: {:?}", start.elapsed());
        eprintln!("Rows in current response: {}", api_response.meta.count);
        eprintln!("Total records: {}", api_response.meta.total_count);
        eprintln!("Total pages: {}", api_response.meta.total_pages);
        eprintln!("Processed {} of {} records", total_rows_processed, api_response.meta.total_count);

        if current_page >= api_response.meta.total_pages {
            break;
        }
        
        current_page += 1;
    }

    if let Some(writer) = writer {
        writer.close()?;
    }
    
    eprintln!("\nFinal Summary:");
    eprintln!("Total execution time: {:?}", start.elapsed());
    eprintln!("Total records processed: {}", total_rows_processed);

    Ok(())
}