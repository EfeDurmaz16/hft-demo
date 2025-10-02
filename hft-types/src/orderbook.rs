use crate::{BookLevel, OrderBook, MarketTick};
use std::collections::HashMap;

/// Order book manager for maintaining level 2 data
pub struct OrderBookManager {
    books: HashMap<String, OrderBook>,
}

impl OrderBookManager {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }

    /// Update order book from market tick (simplified L1 -> L2 conversion)
    pub fn update_from_tick(&mut self, tick: &MarketTick) {
        let book = self.books
            .entry(tick.symbol.clone())
            .or_insert_with(|| OrderBook::new(tick.symbol.clone(), tick.timestamp_nanos));

        book.timestamp_nanos = tick.timestamp_nanos;

        // Simplified: Create synthetic L2 data from L1 tick
        // In production, this would come from actual exchange order book feed
        let spread_bps = 10.0; // 10 basis points
        let spread = tick.price * (spread_bps / 10000.0);

        // Clear existing levels
        book.bids.clear();
        book.asks.clear();

        // Create 5 levels on each side
        for i in 0..5 {
            let bid_price = tick.price - spread / 2.0 - (i as f64 * tick.price * 0.0001);
            let ask_price = tick.price + spread / 2.0 + (i as f64 * tick.price * 0.0001);

            book.bids.push(BookLevel {
                price: bid_price,
                quantity: tick.volume as f64 / (i + 1) as f64,
            });

            book.asks.push(BookLevel {
                price: ask_price,
                quantity: tick.volume as f64 / (i + 1) as f64,
            });
        }
    }

    /// Get order book for symbol
    pub fn get_book(&self, symbol: &str) -> Option<&OrderBook> {
        self.books.get(symbol)
    }

    /// Get all books
    pub fn get_all_books(&self) -> &HashMap<String, OrderBook> {
        &self.books
    }

    /// Get best bid/ask for symbol
    pub fn get_bbo(&self, symbol: &str) -> Option<(f64, f64)> {
        self.books.get(symbol).and_then(|book| {
            book.best_bid()
                .and_then(|bid| book.best_ask().map(|ask| (bid.price, ask.price)))
        })
    }

    /// Calculate VWAP (Volume Weighted Average Price)
    pub fn calculate_vwap(&self, symbol: &str, side_depth: usize) -> Option<f64> {
        self.books.get(symbol).map(|book| {
            let levels = if side_depth > 0 {
                &book.bids[..side_depth.min(book.bids.len())]
            } else {
                &book.bids
            };

            let total_value: f64 = levels.iter()
                .map(|level| level.price * level.quantity)
                .sum();
            let total_quantity: f64 = levels.iter()
                .map(|level| level.quantity)
                .sum();

            if total_quantity > 0.0 {
                total_value / total_quantity
            } else {
                0.0
            }
        })
    }

    /// Check if book is crossed (bid >= ask, indicating arbitrage opportunity)
    pub fn is_crossed(&self, symbol: &str) -> bool {
        if let Some((bid, ask)) = self.get_bbo(symbol) {
            bid >= ask
        } else {
            false
        }
    }

    /// Get market depth (total quantity at each price level)
    pub fn get_depth(&self, symbol: &str, num_levels: usize) -> Option<(Vec<BookLevel>, Vec<BookLevel>)> {
        self.books.get(symbol).map(|book| {
            let bids = book.bids.iter()
                .take(num_levels)
                .cloned()
                .collect();
            let asks = book.asks.iter()
                .take(num_levels)
                .cloned()
                .collect();
            (bids, asks)
        })
    }
}

impl Default for OrderBookManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_orderbook_manager() {
        let mut manager = OrderBookManager::new();

        let tick = MarketTick::new(
            "BTC/USD".to_string(),
            45000.0,
            100,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        );

        manager.update_from_tick(&tick);

        let book = manager.get_book("BTC/USD").unwrap();
        assert_eq!(book.bids.len(), 5);
        assert_eq!(book.asks.len(), 5);

        let (bid, ask) = manager.get_bbo("BTC/USD").unwrap();
        assert!(bid < ask);
        assert!(!manager.is_crossed("BTC/USD"));

        let vwap = manager.calculate_vwap("BTC/USD", 3).unwrap();
        assert!(vwap > 0.0);
    }
}
