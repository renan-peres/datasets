pub const FINNHUB_API_KEY: &str = "ctl0tnpr01qn6d7jqpj0ctl0tnpr01qn6d7jqpjg";

pub fn get_symbols() -> Vec<&'static str> {
    vec![
        // Technology
        "AAPL", "MSFT", "GOOGL", "GOOG", "META", "NVDA", "AVGO", "ORCL", "CRM", "ACN", "ADBE", "CSCO", "INTC",
        // Communication Services
        "NFLX", "DIS", "CMCSA", "VZ", "T",
        // Consumer Discretionary
        "AMZN", "TSLA", "HD", "MCD", "NKE", "SBUX", "TGT", "LOW",
        // Consumer Staples
        "WMT", "PG", "KO", "PEP", "COST",
        // Financials
        "BRK-B", "JPM", "BAC", "WFC", "GS", "MS", "BLK",
        // Healthcare
        "UNH", "JNJ", "PFE", "ABBV", "MRK", "LLY",
        // Industrials
        "CAT", "BA", "HON", "UPS", "RTX", "GE",
        // Energy
        "XOM", "CVX", "COP", "SLB",
        // Materials
        "LIN", "APD", "ECL",
        // Real Estate
        "PLD", "AMT", "CCI"
    ]
}