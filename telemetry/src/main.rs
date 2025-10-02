use anyhow::Result;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use prometheus::{Encoder, Histogram, HistogramOpts, IntCounter, Registry, TextEncoder};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::info;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // Simulated metrics matching feed_handler
    pub static ref TICKS_RECEIVED: IntCounter = IntCounter::new(
        "feed_ticks_received_total",
        "Total number of market ticks received"
    )
    .unwrap();

    pub static ref LATENCY_HISTOGRAM: Histogram = Histogram::with_opts(
        HistogramOpts::new("feed_latency_micros", "Tick processing latency in microseconds")
            .buckets(vec![
                1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0
            ])
    )
    .unwrap();

    pub static ref ORDERS_PLACED: IntCounter = IntCounter::new(
        "gateway_orders_placed_total",
        "Total number of orders placed"
    )
    .unwrap();
}

pub fn init_metrics() {
    REGISTRY.register(Box::new(TICKS_RECEIVED.clone())).unwrap();
    REGISTRY.register(Box::new(LATENCY_HISTOGRAM.clone())).unwrap();
    REGISTRY.register(Box::new(ORDERS_PLACED.clone())).unwrap();
}

#[derive(Debug, Clone, Serialize)]
struct MetricsSnapshot {
    ticks_received: u64,
    orders_placed: u64,
    latency_p50: f64,
    latency_p99: f64,
    latency_mean: f64,
    timestamp: u64,
}

impl MetricsSnapshot {
    fn capture() -> Self {
        let ticks = TICKS_RECEIVED.get();
        let orders = ORDERS_PLACED.get();

        // Get latency histogram metrics
        let hist = LATENCY_HISTOGRAM.get_sample_sum();
        let count = LATENCY_HISTOGRAM.get_sample_count();
        let mean = if count > 0 {
            hist / count as f64
        } else {
            0.0
        };

        // For demo purposes, simulate percentiles
        let p50 = mean * 0.8;
        let p99 = mean * 1.5;

        Self {
            ticks_received: ticks,
            orders_placed: orders,
            latency_p50: p50,
            latency_p99: p99,
            latency_mean: mean,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

// Prometheus metrics endpoint
async fn metrics_handler() -> Response {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Response::builder()
        .header("Content-Type", encoder.format_type())
        .body(buffer.into())
        .unwrap()
}

// WebSocket handler for live metrics
async fn ws_handler(
    ws: WebSocketUpgrade,
    metrics_tx: Arc<broadcast::Sender<MetricsSnapshot>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, metrics_tx))
}

async fn handle_socket(socket: WebSocket, metrics_tx: Arc<broadcast::Sender<MetricsSnapshot>>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = metrics_tx.subscribe();

    // Send initial snapshot
    if let Ok(snapshot) = serde_json::to_string(&MetricsSnapshot::capture()) {
        let _ = sender.send(Message::Text(snapshot)).await;
    }

    // Spawn task to send metrics updates
    let mut send_task = tokio::spawn(async move {
        while let Ok(snapshot) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&snapshot) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages (just for keepalive)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // Echo or ignore
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

// Simulate metric updates for demo
async fn simulate_metrics(tx: broadcast::Sender<MetricsSnapshot>) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    let mut counter = 0u64;

    loop {
        interval.tick().await;
        counter += 1;

        // Simulate incoming ticks
        for _ in 0..100 {
            TICKS_RECEIVED.inc();
            LATENCY_HISTOGRAM.observe(10.0 + (counter % 50) as f64);
        }

        // Simulate orders every 10 iterations
        if counter % 10 == 0 {
            ORDERS_PLACED.inc();
        }

        // Broadcast snapshot
        let snapshot = MetricsSnapshot::capture();
        let _ = tx.send(snapshot);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    init_metrics();

    // Broadcast channel for metrics updates
    let (metrics_tx, _) = broadcast::channel::<MetricsSnapshot>(100);
    let metrics_tx = Arc::new(metrics_tx);

    // Spawn metrics simulator
    let tx_clone = metrics_tx.clone();
    tokio::spawn(async move {
        simulate_metrics((*tx_clone).clone()).await;
    });

    // Build router
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/ws", get({
            let tx = metrics_tx.clone();
            move |ws| ws_handler(ws, tx)
        }))
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:9090";
    info!("Telemetry server running on http://{}", addr);
    info!("  Prometheus: http://{}/metrics", addr);
    info!("  WebSocket:  ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
