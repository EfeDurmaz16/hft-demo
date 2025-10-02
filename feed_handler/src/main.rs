use anyhow::Result;
use crossbeam::channel::{bounded, Sender};
use lazy_static::lazy_static;
use prometheus::{Histogram, HistogramOpts, IntCounter, Registry};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
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

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref TICKS_RECEIVED: IntCounter = IntCounter::new(
        "feed_ticks_received_total",
        "Total number of market ticks received"
    )
    .unwrap();
    pub static ref LATENCY_HISTOGRAM: Histogram = Histogram::with_opts(
        HistogramOpts::new("feed_latency_micros", "Tick processing latency in microseconds")
            .buckets(vec![
                1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0,
                10000.0
            ])
    )
    .unwrap();
}

pub fn init_metrics() {
    REGISTRY
        .register(Box::new(TICKS_RECEIVED.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(LATENCY_HISTOGRAM.clone()))
        .unwrap();
}

struct FeedHandler {
    socket: UdpSocket,
    strategy_tx: Sender<EnrichedTick>,
}

impl FeedHandler {
    async fn new(listen_addr: &str, strategy_tx: Sender<EnrichedTick>) -> Result<Self> {
        let socket = UdpSocket::bind(listen_addr).await?;
        info!("Feed handler listening on {}", listen_addr);

        Ok(Self {
            socket,
            strategy_tx,
        })
    }

    async fn run(&mut self) -> Result<()> {
        let mut buf = vec![0u8; 4096];

        loop {
            let (n, _addr) = self.socket.recv_from(&mut buf).await?;
            let receive_time_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            match serde_json::from_slice::<MarketTick>(&buf[..n]) {
                Ok(tick) => {
                    let latency_nanos = receive_time_nanos - tick.timestamp_nanos;
                    let latency_micros = latency_nanos as f64 / 1000.0;

                    // Update metrics
                    TICKS_RECEIVED.inc();
                    LATENCY_HISTOGRAM.observe(latency_micros);

                    let enriched = EnrichedTick {
                        tick,
                        receive_time_nanos,
                        latency_micros,
                    };

                    // Forward to strategy engine (non-blocking)
                    if let Err(e) = self.strategy_tx.try_send(enriched) {
                        warn!("Strategy channel full or disconnected: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Failed to parse tick: {}", e);
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

    init_metrics();

    let listen_addr = "127.0.0.1:9001";

    // Create bounded channel to strategy engine (lock-free, high throughput)
    let (strategy_tx, strategy_rx) = bounded::<EnrichedTick>(100_000);

    // Spawn strategy consumer in separate thread
    let registry = Arc::new(REGISTRY.clone());
    std::thread::spawn(move || {
        strategy_consumer(strategy_rx, registry);
    });

    let mut handler = FeedHandler::new(listen_addr, strategy_tx).await?;
    handler.run().await?;

    Ok(())
}

fn strategy_consumer(
    rx: crossbeam::channel::Receiver<EnrichedTick>,
    _registry: Arc<Registry>,
) {
    info!("Strategy consumer started");

    for enriched in rx.iter() {
        // Here we would send to strategy_engine over IPC/channel
        // For this demo, we'll just log occasionally
        if enriched.tick.volume > 90 {
            tracing::debug!(
                "High volume tick: {} @ {} (latency: {:.2}µs)",
                enriched.tick.symbol,
                enriched.tick.price,
                enriched.latency_micros
            );
        }
    }
}
