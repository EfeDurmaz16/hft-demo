pub mod messaging;
pub mod orderbook;
pub mod replay;
pub mod strategies;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Market tick data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub symbol: String,
    pub price: f64,
    pub volume: u64,
    pub timestamp_nanos: u128,
}

impl MarketTick {
    pub fn new(symbol: String, price: f64, volume: u64, timestamp_nanos: u128) -> Self {
        Self {
            symbol,
            price,
            volume,
            timestamp_nanos,
        }
    }
}

/// Enriched tick with latency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedTick {
    pub tick: MarketTick,
    pub receive_time_nanos: u128,
    pub latency_micros: f64,
}

/// Trading order side
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

/// Trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub timestamp_nanos: u128,
}

impl Order {
    pub fn new(
        order_id: u64,
        symbol: String,
        side: OrderSide,
        price: f64,
        quantity: f64,
        timestamp_nanos: u128,
    ) -> Self {
        Self {
            order_id,
            symbol,
            side,
            price,
            quantity,
            timestamp_nanos,
        }
    }
}

/// Order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: f64,
    pub quantity: f64,
}

/// Level 2 Order Book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
    pub timestamp_nanos: u128,
}

impl OrderBook {
    pub fn new(symbol: String, timestamp_nanos: u128) -> Self {
        Self {
            symbol,
            bids: Vec::new(),
            asks: Vec::new(),
            timestamp_nanos,
        }
    }

    pub fn best_bid(&self) -> Option<&BookLevel> {
        self.bids.first()
    }

    pub fn best_ask(&self) -> Option<&BookLevel> {
        self.asks.first()
    }

    pub fn spread(&self) -> Option<f64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some((ask.price + bid.price) / 2.0),
            _ => None,
        }
    }
}

/// Trading signal from strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub signal_type: SignalType,
    pub timestamp_nanos: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    Threshold,
    MarketMaking,
    Arbitrage,
    MeanReversion,
}

/// Configuration for market symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolConfig {
    pub symbol: String,
    pub tick_size: f64,
    pub lot_size: f64,
    pub min_price: f64,
    pub max_price: f64,
}

/// Error types
#[derive(Debug, thiserror::Error)]
pub enum HftError {
    #[error("Invalid price: {0}")]
    InvalidPrice(f64),

    #[error("Invalid quantity: {0}")]
    InvalidQuantity(f64),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Order book empty for symbol: {0}")]
    OrderBookEmpty(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type HftResult<T> = Result<T, HftError>;
