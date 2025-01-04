import requests
import polars as pl
from typing import List, Dict, Any
import os
from datetime import datetime, timedelta
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Get API key from environment variables
ALPHAVANTAGE_API_KEY = os.getenv('ALPHAVANTAGE_API_KEY')
if not ALPHAVANTAGE_API_KEY:
    raise ValueError("ALPHAVANTAGE_API_KEY not found in environment variables")

def fetch_news_data(api_key, categories, time_from):
    all_articles = []

    for category in categories:
        url = f"https://www.alphavantage.co/query?function=NEWS_SENTIMENT&topics={category}&time_from={time_from}&apikey={api_key}"
        print(f"Fetching news for category: {category} from {time_from}")

        response = requests.get(url)
        news_data = response.json()

        if 'feed' in news_data:
            for article in news_data['feed']:
                time_published = article.get('time_published', '19700101T000000')
                try:
                    timestamp = datetime.strptime(time_published, "%Y%m%dT%H%M%S").timestamp() * 1000
                except ValueError:
                    timestamp = datetime.strptime(time_published + "00", "%Y%m%dT%H%M%S").timestamp() * 1000

                all_articles.append({
                    "category": category,
                    "title": article.get("title", ""),
                    "source": article.get("source", ""),
                    "summary": article.get("summary", ""),
                    "url": article.get("url", ""),
                    "time_published": timestamp,
                    "sentiment_score": article.get("overall_sentiment_score", 0.0),
                    "sentiment_label": article.get("overall_sentiment_label", "")
                })

    return all_articles

def main():
    api_key = ALPHAVANTAGE_API_KEY
    categories = [
        "mergers_and_acquisitions", "earnings", "financial_markets", "finance", 
        "real_estate", "ipo", "economy_monetary", "economy_fiscal", "economy_macro",
        "blockchain", "energy_transportation", "retail_wholesale", "technology"
    ]

    two_days_ago = datetime.utcnow() - timedelta(days=2)
    time_from = two_days_ago.strftime("%Y%m%dT%H%M")

    articles = fetch_news_data(api_key, categories, time_from)

    df = pl.DataFrame(articles)
    
    date = datetime.today().strftime('%Y-%m-%d')
    df.write_parquet(f"../../data/finance/market_news_{date}.parquet")

    print("\nFinal Summary:")
    print(f"Total articles processed: {len(articles)}")
    print(f"Time range: from {time_from} to now")

if __name__ == "__main__":
    main()