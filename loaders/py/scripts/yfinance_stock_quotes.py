import yfinance as yf
import polars as pl
import pandas as pd
from datetime import datetime
import time
from typing import List, Dict
import os

def load_symbols(file_path: str) -> List[str]:
    """Load symbols from a text file"""
    with open(file_path, 'r') as f:
        return [line.strip() for line in f if line.strip()]

def download_stock_data(symbols: List[str], start_date: str, end_date: str) -> Dict[str, pl.DataFrame]:
    all_data = {}
    total = len(symbols)
    
    print(f"Starting download of {total} symbols...")
    
    for idx, symbol in enumerate(symbols, 1):
        try:
            print(f"[{idx}/{total}] Downloading {symbol}...", end='\r')
            df = yf.download(symbol, 
                           start=start_date, 
                           end=end_date,
                           progress=False)
            
            df = df.reset_index()
            
            if isinstance(df.columns, pd.MultiIndex):
                df.columns = [col[0] for col in df.columns]
            
            df_pol = pl.from_pandas(df)
            df_pol = df_pol.with_columns(pl.lit(symbol).alias('Symbol'))
            all_data[symbol] = df_pol
            
            time.sleep(0.1)
            
        except Exception as e:
            print(f"\nError downloading {symbol}: {str(e)}")
            continue
    
    print("\nDownload completed!")
    return all_data

def combine_and_save_data(data_dict: Dict[str, pl.DataFrame], output_file: str) -> pl.DataFrame:
    print("Combining data...")
    combined_df = pl.concat(data_dict.values())
    combined_df = combined_df.sort(['Symbol', 'Date'])
    
    print(f"Saving data to {output_file}...")
    combined_df.write_parquet(output_file)
    
    return combined_df

def create_output_directory(directory: str):
    if not os.path.exists(directory):
        os.makedirs(directory)
        print(f"Created directory: {directory}")

def main():
    output_dir = "../../data/finance"
    create_output_directory(output_dir)
    
    symbols_file = 'tickers.txt'
    try:
        symbols = load_symbols(symbols_file)
        print(f"Loaded {len(symbols)} symbols from {symbols_file}")
    except Exception as e:
        print(f"Error loading symbols: {str(e)}")
        return
    
    start_date = '2020-01-01'
    end_date = datetime.today().strftime('%Y-%m-%d')
    output_file = os.path.join(output_dir, f'historical_stock_quotes_{start_date}_to_{end_date}.parquet')
    
    data_dict = download_stock_data(symbols, start_date, end_date)
    
    if data_dict:
        final_df = combine_and_save_data(data_dict, output_file)
        print(f"\nData saved successfully! Shape: {final_df.shape}")
        
        print("\nSummary:")
        print(f"Total symbols downloaded: {len(data_dict)}")
        print(f"Date range: {start_date} to {end_date}")
        print(f"Total rows: {final_df.shape[0]}")
        print(f"Output file: {output_file}")
        
        memory_mb = final_df.estimated_size() / (1024 * 1024)
        print(f"Memory usage: {memory_mb:.2f} MB")
    else:
        print("No data was downloaded!")

if __name__ == "__main__":
    main()
