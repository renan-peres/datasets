import WebSocket from 'ws';

const socket = new WebSocket('wss://ws.finnhub.io?token=ctl0tnpr01qn6d7jqpj0ctl0tnpr01qn6d7jqpjg');

// Array of assets to track (stocks and crypto)
const assets = [
    'META', 
    'GOOGL', 
    'NFLX', 
    'AAPL',
    'BINANCE:BTCUSDT',
    'IC MARKETS:1'
];

// Connection opened -> Subscribe
socket.on('open', function () {
    // Subscribe to each asset
    assets.forEach(asset => {
        socket.send(JSON.stringify({'type':'subscribe', 'symbol': asset}));
        console.log(`Subscribed to ${asset}`);
    });
});

// Listen for messages
socket.on('message', function (data) {
    try {
        const parsedData = JSON.parse(data.toString());
        if (parsedData.data) {
            parsedData.data.forEach(trade => {
                // Format price to 2 decimal places for stocks, 4 for crypto
                const price = trade.s.includes('BINANCE') ? trade.p.toFixed(4) : trade.p.toFixed(2);
                console.log(`${trade.s}: $${price} | Volume: ${trade.v} | Timestamp: ${new Date(trade.t).toLocaleTimeString()}`);
            });
        }
    } catch (error) {
        console.error('Error parsing message:', error);
    }
});

// Handle errors
socket.on('error', function (error) {
    console.error('WebSocket error: ', error);
});

// Unsubscribe function
const unsubscribe = function(symbol) {
    socket.send(JSON.stringify({'type':'unsubscribe','symbol': symbol}));
    console.log(`Unsubscribed from ${symbol}`);
};

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('\nClosing connections...');
    assets.forEach(asset => unsubscribe(asset));
    setTimeout(() => {
        socket.close();
        process.exit();
    }, 1000);
});