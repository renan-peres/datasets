import requests
import polars as pl
import base64
import os
import sys
from datetime import datetime
from dotenv import load_dotenv
import re

# Load environment variables
load_dotenv()

# Add the directory containing make_clean_names.py to the Python path
sys.path.append(os.path.abspath('../'))

# Import the make_clean_names function
from make_clean_names import make_clean_names

def fetch_access_token(client_id, client_secret):
    auth_string = base64.b64encode(f"{client_id}:{client_secret}".encode()).decode()
    url = "https://ews.fip.finra.org/fip/rest/ews/oauth2/access_token?grant_type=client_credentials"
    headers = {
        "Authorization": f"Basic {auth_string}"
    }
    response = requests.post(url, headers=headers)
    if response.status_code != 200:
        print(f"Error fetching access token: {response.status_code} {response.text}")
    response.raise_for_status()
    return response.json()["access_token"]

def fetch_market_data(access_token):
    url = "https://api.finra.org/data/group/fixedIncomeMarket/name/treasuryMonthlyAggregates"
    headers = {
        "Authorization": f"Bearer {access_token}",
        "Accept": "application/json"
    }
    params = {
        "limit": "10000"
    }
    response = requests.get(url, headers=headers, params=params)
    if response.status_code != 200:
        print(f"Error fetching market data: {response.status_code} {response.text}")
    response.raise_for_status()
    return response.json()

def main():
    client_id = os.getenv("FINRA_CLIENT_ID")
    client_secret = os.getenv("FINRA_CLIENT_SECRET")
    
    if not client_id or not client_secret:
        print("Client ID or Client Secret not found in environment variables.")
        return
    
    access_token = fetch_access_token(client_id, client_secret)
    market_data = fetch_market_data(access_token)
    
    if market_data:        
        # Create Polars DataFrame
        df = pl.DataFrame(market_data)
        df = make_clean_names(df)
        
        # Ensure the directory exists
        output_dir = "../../../data/finance"
        os.makedirs(output_dir, exist_ok=True)
        
        # Write to CSV
        date = datetime.today().strftime('%Y-%m-%d')
        df.write_parquet(f"{output_dir}/treasury_monthly_aggregates_{date}.parquet")
        
        print(f"Data written to {output_dir}/treasury_monthly_aggregates_{date}.parquet")
        print(f"Total rows processed: {len(market_data)}")
    else:
        print("No market data found.")

if __name__ == "__main__":
    main()