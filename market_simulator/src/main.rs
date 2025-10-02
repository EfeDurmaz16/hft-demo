use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketTick {
    pub symbol: String,
    pub price: f64,
    pub volume: u64,
    pub timestamp_nanos: u128,
}

impl MarketTick {
    pub fn new(symbol: String, price: f64, volume: u64) -> Self {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        Self {
            symbol,
            price,
            volume,
            timestamp_nanos,
        }
    }
}

struct MarketSimulator {
    socket: UdpSocket,
    symbols: Vec<String>,
    base_prices: Vec<f64>,
}

impl MarketSimulator {
    async fn new(bind_addr: &str, target_addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        socket.connect(target_addr).await?;

        info!("Market simulator bound to {} → {}", bind_addr, target_addr);

        Ok(Self {
            socket,
            symbols: vec![
                "BTC/USD".to_string(),
                "ETH/USD".to_string(),
                "SOL/USD".to_string(),
                "AVAX/USD".to_string(),
            ],
            base_prices: vec![45000.0, 2500.0, 100.0, 25.0],
        })
    }

    async fn run(&mut self, ticks_per_second: u64) -> Result<()> {
        let interval_micros = 1_000_000 / ticks_per_second;
        let mut ticker = interval(Duration::from_micros(interval_micros));
        let mut rng = rand::thread_rng();

        info!("Generating {} ticks/second", ticks_per_second);

        loop {
            ticker.tick().await;

            // Pick random symbol
            let idx = rng.gen_range(0..self.symbols.len());
            let symbol = self.symbols[idx].clone();
            let base_price = self.base_prices[idx];

            // Add random walk
            let price_delta = rng.gen_range(-0.01..0.01);
            let price = base_price * (1.0 + price_delta);
            let volume = rng.gen_range(1..100);

            let tick = MarketTick::new(symbol, price, volume);
            let payload = serde_json::to_vec(&tick)?;

            match self.socket.send(&payload).await {
                Ok(n) => {
                    tracing::debug!("Sent {} bytes: {:?}", n, tick);
                }
                Err(e) => {
                    warn!("Failed to send tick: {}", e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let bind_addr = "0.0.0.0:0"; // ephemeral port
    let target_addr = "127.0.0.1:9001"; // feed_handler listens here
    let ticks_per_second = 10_000; // 10k ticks/sec for high-frequency demo

    let mut simulator = MarketSimulator::new(bind_addr, target_addr).await?;
    simulator.run(ticks_per_second).await?;

    Ok(())
}
