use anyhow::Result;
use hft_types::MarketTick;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

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

            // Random walk
            let price_delta = rng.gen_range(-0.01..0.01);
            let price = base_price * (1.0 + price_delta);
            let volume = rng.gen_range(1..100);

            let timestamp_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_nanos();

            let tick = MarketTick::new(symbol, price, volume, timestamp_nanos);
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

    let bind_addr = "0.0.0.0:0";
    let target_addr = "127.0.0.1:9001";
    let ticks_per_second = 10_000;

    let mut simulator = MarketSimulator::new(bind_addr, target_addr).await?;
    simulator.run(ticks_per_second).await?;

    Ok(())
}
