import asyncio
import aiohttp
import json
import polars as pl
from typing import List, Dict, Any
import os
from datetime import datetime
import logging
from make_clean_names import make_clean_names
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Get API key from environment variables
FINNHUB_API_KEY = os.getenv('FINNHUB_API_KEY')
if not FINNHUB_API_KEY:
    raise ValueError("FINNHUB_API_KEY not found in environment variables")

async def fetch_company_profile(
    session: aiohttp.ClientSession,
    symbol: str,
    api_key: str,
    semaphore: asyncio.Semaphore,
    retry_count: int = 3
) -> Dict[str, Any]:
    """
    Fetch company profile for a single symbol with retry logic and rate limiting.
    """
    async with semaphore:
        for attempt in range(retry_count):
            try:
                async with session.get(
                    "https://finnhub.io/api/v1/stock/profile2",
                    params={"token": api_key, "symbol": symbol},
                    timeout=5
                ) as response:
                    if response.status == 429:
                        logger.warning(f"Rate limit hit for {symbol}, waiting...")
                        await asyncio.sleep(1)
                        continue
                    
                    if response.status != 200:
                        logger.error(f"Error response for {symbol}: {await response.text()}")
                        return None
                    
                    text = await response.text()
                    if not text.strip():
                        logger.warning(f"Empty response for symbol: {symbol}")
                        return None
                    
                    data = json.loads(text)
                    if not data:
                        logger.warning(f"Empty profile data for symbol: {symbol}")
                        return None
                    
                    data['symbol'] = symbol
                    logger.info(f"Successfully processed profile for: {symbol}")
                    return data
                    
            except asyncio.TimeoutError:
                logger.warning(f"Timeout fetching {symbol}")
                if attempt == retry_count - 1:
                    return None
            except Exception as e:
                logger.error(f"Error fetching {symbol}: {str(e)}")
                if attempt == retry_count - 1:
                    return None

async def fetch_all_profiles(symbols: List[str], api_key: str) -> List[Dict[str, Any]]:
    """Fetch profiles for multiple symbols with optimized concurrency."""
    connector = aiohttp.TCPConnector(limit=50)
    semaphore = asyncio.Semaphore(30)
    
    async with aiohttp.ClientSession(
        connector=connector,
        timeout=aiohttp.ClientTimeout(total=30)
    ) as session:
        # Process symbols in chunks to avoid rate limiting
        chunk_size = 10
        all_profiles = []
        
        for i in range(0, len(symbols), chunk_size):
            chunk = symbols[i:i + chunk_size]
            tasks = [
                fetch_company_profile(session, symbol, api_key, semaphore)
                for symbol in chunk
            ]
            chunk_results = await asyncio.gather(*tasks)
            all_profiles.extend([p for p in chunk_results if p is not None])
            await asyncio.sleep(0.5)  # Wait between chunks
            
        return all_profiles

def load_symbols(file_path: str) -> List[str]:
    """Load symbols from a text file"""
    with open(file_path, 'r') as f:
        return [line.strip() for line in f if line.strip()]

def save_profiles(profiles: List[Dict[str, Any]], output_file: str):
    """Convert profiles to Polars DataFrame, clean column names, and save to parquet"""
    if not profiles:
        logger.error("No profiles to save!")
        return None
    
    # Convert to Polars DataFrame
    df = pl.DataFrame(profiles)
    
    # Clean column names
    df = make_clean_names(
        df,
        case_type="snake",
        strip_accents=True,
        strip_underscores="both",
        insert_underscores=True
    )
    
    # Save to parquet
    df.write_parquet(output_file)
    logger.info(f"Saved {len(profiles)} profiles to {output_file}")
    return df

def create_output_directory(directory: str):
    """Create output directory if it doesn't exist"""
    if not os.path.exists(directory):
        os.makedirs(directory)
        logger.info(f"Created directory: {directory}")

async def main():
    output_dir = "../../finance/stock_data"
    create_output_directory(output_dir)
    
    symbols_file = 'tickers.txt'
    try:
        symbols = load_symbols(symbols_file)
        logger.info(f"Loaded {len(symbols)} symbols from {symbols_file}")
    except Exception as e:
        logger.error(f"Error loading symbols: {str(e)}")
        return
    
    current_date = datetime.today().strftime('%Y-%m-%d')
    output_file = os.path.join(output_dir, f'company_profiles.parquet')
    
    profiles = await fetch_all_profiles(symbols, FINNHUB_API_KEY)
    
    if profiles:
        df = save_profiles(profiles, output_file)
        if df is not None:
            logger.info("\nSummary:")
            logger.info(f"Total profiles downloaded: {len(profiles)}")
            logger.info(f"Total columns: {df.shape[1]}")
            logger.info(f"Output file: {output_file}")
            
            memory_mb = df.estimated_size() / (1024 * 1024)
            logger.info(f"Memory usage: {memory_mb:.2f} MB")
    else:
        logger.error("No profiles were downloaded!")

if __name__ == "__main__":
    asyncio.run(main())