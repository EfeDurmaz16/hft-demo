use crate::{EnrichedTick, OrderSide, TradingSignal, SignalType};
use std::collections::HashMap;

/// Base strategy trait
pub trait Strategy: Send {
    fn process_tick(&mut self, tick: &EnrichedTick) -> Option<TradingSignal>;
    fn name(&self) -> &str;
}

/// Simple threshold-based strategy
pub struct ThresholdStrategy {
    thresholds: HashMap<String, (f64, f64)>,
    order_size: f64,
}

impl ThresholdStrategy {
    pub fn new(thresholds: HashMap<String, (f64, f64)>, order_size: f64) -> Self {
        Self { thresholds, order_size }
    }
}

impl Strategy for ThresholdStrategy {
    fn process_tick(&mut self, enriched: &EnrichedTick) -> Option<TradingSignal> {
        let tick = &enriched.tick;

        if let Some(&(low, high)) = self.thresholds.get(&tick.symbol) {
            let side = if tick.price < low {
                Some(OrderSide::Buy)
            } else if tick.price > high {
                Some(OrderSide::Sell)
            } else {
                None
            };

            side.map(|s| TradingSignal {
                symbol: tick.symbol.clone(),
                side: s,
                price: tick.price,
                quantity: self.order_size,
                signal_type: SignalType::Threshold,
                timestamp_nanos: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
            })
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "ThresholdStrategy"
    }
}

/// Market making strategy
pub struct MarketMakingStrategy {
    spread_bps: f64, // Spread in basis points
    order_size: f64,
    last_prices: HashMap<String, f64>,
}

impl MarketMakingStrategy {
    pub fn new(spread_bps: f64, order_size: f64) -> Self {
        Self {
            spread_bps,
            order_size,
            last_prices: HashMap::new(),
        }
    }
}

impl Strategy for MarketMakingStrategy {
    fn process_tick(&mut self, enriched: &EnrichedTick) -> Option<TradingSignal> {
        let tick = &enriched.tick;
        self.last_prices.insert(tick.symbol.clone(), tick.price);

        // Simplified: Place both bid and ask orders (return buy signal for demo)
        let half_spread = tick.price * (self.spread_bps / 10000.0);

        Some(TradingSignal {
            symbol: tick.symbol.clone(),
            side: OrderSide::Buy,
            price: tick.price - half_spread,
            quantity: self.order_size,
            signal_type: SignalType::MarketMaking,
            timestamp_nanos: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        })
    }

    fn name(&self) -> &str {
        "MarketMakingStrategy"
    }
}

/// Mean reversion strategy
pub struct MeanReversionStrategy {
    window_size: usize,
    std_dev_threshold: f64,
    order_size: f64,
    price_history: HashMap<String, Vec<f64>>,
}

impl MeanReversionStrategy {
    pub fn new(window_size: usize, std_dev_threshold: f64, order_size: f64) -> Self {
        Self {
            window_size,
            std_dev_threshold,
            order_size,
            price_history: HashMap::new(),
        }
    }

    fn calculate_mean(&self, prices: &[f64]) -> f64 {
        prices.iter().sum::<f64>() / prices.len() as f64
    }

    fn calculate_std_dev(&self, prices: &[f64], mean: f64) -> f64 {
        let variance = prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        variance.sqrt()
    }
}

impl Strategy for MeanReversionStrategy {
    fn process_tick(&mut self, enriched: &EnrichedTick) -> Option<TradingSignal> {
        let tick = &enriched.tick;
        let history = self.price_history
            .entry(tick.symbol.clone())
            .or_insert_with(Vec::new);

        history.push(tick.price);
        if history.len() > self.window_size {
            history.remove(0);
        }

        if history.len() < self.window_size {
            return None;
        }

        let history_clone = history.clone();
        let mean = self.calculate_mean(&history_clone);
        let std_dev = self.calculate_std_dev(&history_clone, mean);
        let z_score = (tick.price - mean) / std_dev;

        if z_score.abs() > self.std_dev_threshold {
            let side = if z_score > 0.0 {
                OrderSide::Sell // Price too high, sell
            } else {
                OrderSide::Buy // Price too low, buy
            };

            Some(TradingSignal {
                symbol: tick.symbol.clone(),
                side,
                price: tick.price,
                quantity: self.order_size,
                signal_type: SignalType::MeanReversion,
                timestamp_nanos: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
            })
        } else {
            None
        }
    }

    fn name(&self) -> &str {
        "MeanReversionStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarketTick;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_threshold_strategy() {
        let mut thresholds = HashMap::new();
        thresholds.insert("BTC/USD".to_string(), (44000.0, 46000.0));

        let mut strategy = ThresholdStrategy::new(thresholds, 1.0);

        let tick = MarketTick::new(
            "BTC/USD".to_string(),
            43500.0,
            100,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        );

        let enriched = EnrichedTick {
            tick,
            receive_time_nanos: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            latency_micros: 10.0,
        };

        let signal = strategy.process_tick(&enriched);
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().side, OrderSide::Buy);
    }

    #[test]
    fn test_mean_reversion_strategy() {
        let mut strategy = MeanReversionStrategy::new(5, 1.5, 1.0);

        // Add some prices to build history
        for price in [45000.0, 45100.0, 45000.0, 45050.0, 45000.0] {
            let tick = MarketTick::new(
                "BTC/USD".to_string(),
                price,
                100,
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            );

            let enriched = EnrichedTick {
                tick,
                receive_time_nanos: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
                latency_micros: 10.0,
            };

            let _ = strategy.process_tick(&enriched);
        }

        // Now add an outlier
        let tick = MarketTick::new(
            "BTC/USD".to_string(),
            50000.0, // Much higher outlier
            100,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        );

        let enriched = EnrichedTick {
            tick,
            receive_time_nanos: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            latency_micros: 10.0,
        };

        let signal = strategy.process_tick(&enriched);
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().side, OrderSide::Sell);
    }
}
