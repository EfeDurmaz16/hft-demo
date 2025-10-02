use anyhow::Result;
use crossbeam::channel::{bounded, Receiver, Sender};
use lazy_static::lazy_static;
use prometheus::{IntCounter, Registry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketTick {
    pub symbol: String,
    pub price: f64,
    pub volume: u64,
    pub timestamp_nanos: u128,
}

#[derive(Debug, Clone)]
pub struct EnrichedTick {
    pub tick: MarketTick,
    pub receive_time_nanos: u128,
    pub latency_micros: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub timestamp_nanos: u128,
}

#[derive(Debug, Clone, Serialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref SIGNALS_GENERATED: IntCounter = IntCounter::new(
        "strategy_signals_generated_total",
        "Total number of trading signals generated"
    )
    .unwrap();
    pub static ref ORDERS_SENT: IntCounter = IntCounter::new(
        "strategy_orders_sent_total",
        "Total number of orders sent to gateway"
    )
    .unwrap();
}

pub fn init_metrics() {
    REGISTRY
        .register(Box::new(SIGNALS_GENERATED.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(ORDERS_SENT.clone()))
        .unwrap();
}

struct SimpleStrategy {
    // Threshold strategy: if price > high_threshold -> SELL, if price < low_threshold -> BUY
    thresholds: HashMap<String, (f64, f64)>, // (low, high)
    order_tx: Sender<Order>,
}

impl SimpleStrategy {
    fn new(order_tx: Sender<Order>) -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("BTC/USD".to_string(), (44000.0, 46000.0));
        thresholds.insert("ETH/USD".to_string(), (2400.0, 2600.0));
        thresholds.insert("SOL/USD".to_string(), (95.0, 105.0));
        thresholds.insert("AVAX/USD".to_string(), (24.0, 26.0));

        Self {
            thresholds,
            order_tx,
        }
    }

    fn process_tick(&mut self, enriched: EnrichedTick) {
        let tick = enriched.tick;

        if let Some(&(low, high)) = self.thresholds.get(&tick.symbol) {
            let signal = if tick.price < low {
                Some(OrderSide::Buy)
            } else if tick.price > high {
                Some(OrderSide::Sell)
            } else {
                None
            };

            if let Some(side) = signal {
                SIGNALS_GENERATED.inc();

                let order = Order {
                    symbol: tick.symbol.clone(),
                    side,
                    price: tick.price,
                    quantity: 1.0,
                    timestamp_nanos: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos(),
                };

                match self.order_tx.try_send(order.clone()) {
                    Ok(_) => {
                        ORDERS_SENT.inc();
                        info!(
                            "Order sent: {:?} {} @ {}",
                            order.side, order.symbol, order.price
                        );
                    }
                    Err(e) => {
                        warn!("Failed to send order: {}", e);
                    }
                }
            }
        }
    }

    fn run(&mut self, tick_rx: Receiver<EnrichedTick>) {
        info!("Strategy engine started");

        for enriched in tick_rx.iter() {
            self.process_tick(enriched);
        }
    }
}

// In a real system, this would receive from feed_handler via IPC
// For this demo, we'll simulate receiving ticks
fn mock_tick_generator(tx: Sender<EnrichedTick>) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut counter = 0u64;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let symbols = ["BTC/USD", "ETH/USD", "SOL/USD", "AVAX/USD"];
        let prices = [
            43900.0 + (counter % 300) as f64,
            2380.0 + (counter % 300) as f64,
            94.0 + (counter % 15) as f64,
            23.5 + (counter % 4) as f64,
        ];

        for (i, symbol) in symbols.iter().enumerate() {
            let tick = MarketTick {
                symbol: symbol.to_string(),
                price: prices[i],
                volume: counter % 100,
                timestamp_nanos: timestamp - 1000,
            };

            let enriched = EnrichedTick {
                tick,
                receive_time_nanos: timestamp,
                latency_micros: 1.0,
            };

            if tx.send(enriched).is_err() {
                break;
            }
        }

        counter += 1;
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    init_metrics();

    // Channel from feed_handler (simulated)
    let (tick_tx, tick_rx) = bounded::<EnrichedTick>(100_000);

    // Channel to order_gateway
    let (order_tx, order_rx) = bounded::<Order>(10_000);

    // Spawn mock tick generator (in production, this would be feed_handler)
    std::thread::spawn(move || {
        mock_tick_generator(tick_tx);
    });

    // Spawn order consumer (in production, this would send to order_gateway)
    std::thread::spawn(move || {
        for _order in order_rx.iter() {
            // Orders are already logged by strategy, just consume here
        }
    });

    // Run strategy
    let mut strategy = SimpleStrategy::new(order_tx);
    strategy.run(tick_rx);

    Ok(())
}
