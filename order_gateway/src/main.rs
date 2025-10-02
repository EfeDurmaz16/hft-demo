use anyhow::Result;
use lazy_static::lazy_static;
use prometheus::{IntCounter, Registry};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
    pub timestamp_nanos: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref ORDERS_PLACED: IntCounter = IntCounter::new(
        "gateway_orders_placed_total",
        "Total number of orders placed"
    )
    .unwrap();
}

pub fn init_metrics() {
    REGISTRY
        .register(Box::new(ORDERS_PLACED.clone()))
        .unwrap();
}

struct OrderGateway {
    order_id: u64,
}

impl OrderGateway {
    fn new() -> Self {
        Self { order_id: 0 }
    }

    fn place_order(&mut self, order: Order) {
        self.order_id += 1;

        let placed_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let latency_micros = (placed_time - order.timestamp_nanos) as f64 / 1000.0;

        info!(
            "ORDER PLACED [{}]: {:?} {} x {} @ {} (latency: {:.2}µs)",
            self.order_id, order.side, order.quantity, order.symbol, order.price, latency_micros
        );

        ORDERS_PLACED.inc();
    }
}

// Simulated order receiver (in production, this would receive from strategy_engine)
fn mock_order_generator() -> Vec<Order> {
    vec![
        Order {
            symbol: "BTC/USD".to_string(),
            side: OrderSide::Buy,
            price: 43900.0,
            quantity: 0.1,
            timestamp_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        },
        Order {
            symbol: "ETH/USD".to_string(),
            side: OrderSide::Sell,
            price: 2650.0,
            quantity: 1.0,
            timestamp_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        },
    ]
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    init_metrics();

    let mut gateway = OrderGateway::new();

    info!("Order Gateway started - waiting for orders...");

    // Simulate receiving orders
    let orders = mock_order_generator();
    for order in orders {
        gateway.place_order(order);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Keep running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
