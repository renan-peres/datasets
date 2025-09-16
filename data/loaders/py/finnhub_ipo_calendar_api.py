import requests
import polars as pl
from datetime import datetime, timedelta
import re
from dotenv import load_dotenv
import os
import sys

sys.path.append('..')
from make_clean_names import make_clean_names

# Load environment variables
load_dotenv()

# Get API key from environment variables
FINNHUB_API_KEY = os.getenv('FINNHUB_API_KEY')
if not FINNHUB_API_KEY:
    raise ValueError("FINNHUB_API_KEY not found in environment variables")

def fetch_ipo_data(api_key, from_date, to_date):
    url = "https://finnhub.io/api/v1/calendar/ipo"
    params = {
        "token": api_key,
        "from": from_date,
        "to": to_date
    }
    response = requests.get(url, params=params)
    response.raise_for_status()
    return response.json()

def main():
    api_key = os.getenv("FINNHUB_API_KEY")
    
    now = datetime.now()
    from_date = now.strftime("%Y-%m-%d")
    to_date = (now + timedelta(days=90)).strftime("%Y-%m-%d")
    
    data = fetch_ipo_data(api_key, from_date, to_date)
    ipo_calendar = data.get("ipoCalendar", [])
    
    if ipo_calendar:
        # Create Polars DataFrame
        df = pl.DataFrame(ipo_calendar)
        
        # Clean column names
        df = make_clean_names(df)
        
        # Ensure the directory exists
        output_dir = "../../data/finance"
        os.makedirs(output_dir, exist_ok=True)
        
        # Write to Parquet        
        date = datetime.today().strftime('%Y-%m-%d')
        df.write_parquet(f"{output_dir}/ipo_calendar.parquet")

        print(f"Data written to {output_dir}/ipo_calendar.parquet")
        print(f"Date range: {from_date} to {to_date}")
        print(f"Total IPOs found: {len(ipo_calendar)}")
    else:
        print("No IPO data found.")

if __name__ == "__main__":
    main()