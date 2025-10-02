# HFT Demo - High-Frequency Trading Low-Latency System

A complete **end-to-end high-frequency trading simulation** built in Rust with a Next.js dashboard. This demo showcases ultra-low-latency market data processing, strategy execution, and real-time telemetry.

## 🏗️ Architecture

```
┌─────────────────┐      UDP       ┌──────────────┐
│ Market          │───────────────>│ Feed         │
│ Simulator       │   10k ticks/s  │ Handler      │
└─────────────────┘                └──────────────┘
                                           │
                                    Crossbeam Channel
                                           │
                                           ▼
                                    ┌──────────────┐
                                    │ Strategy     │
                                    │ Engine       │
                                    └──────────────┘
                                           │
                                           ▼
                                    ┌──────────────┐
                                    │ Order        │
                                    │ Gateway      │
                                    └──────────────┘

        ┌────────────────────────────────────────┐
        │         Telemetry Service              │
        │  (Prometheus + WebSocket)              │
        └────────────────────────────────────────┘
                         │
                    WebSocket
                         │
                         ▼
        ┌────────────────────────────────────────┐
        │         Web Dashboard (Next.js)        │
        │  - Latency Charts (p50, p99)           │
        │  - Tick Volume                         │
        │  - Order Count                         │
        └────────────────────────────────────────┘
```

## 📦 Components

### Rust Microservices

1. **hft-types** - Shared types library with:
   - Core data structures (MarketTick, Order, OrderBook)
   - IPC messaging framework with TCP framing
   - Market replay system for backtesting
   - Strategy framework (Threshold, Market Making, Mean Reversion)
   - Order book manager with L2 data reconstruction

2. **market_simulator** - Generates fake market ticks over UDP at 10k/sec
3. **feed_handler** - Receives UDP ticks, measures latency, forwards to strategy
4. **strategy_engine** - Multiple trading strategies available:
   - Threshold-based strategy
   - Market making with spread management
   - Mean reversion with statistical analysis
5. **order_gateway** - Simulates order placement with latency tracking
6. **telemetry** - Prometheus metrics + WebSocket feed for dashboard

### Frontend

- **web-ui** - Next.js 14 dashboard with Recharts for live visualization

### Infrastructure

- **Prometheus** - Time-series metrics storage
- **Grafana** - Advanced metrics visualization

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Node.js 18+ (`brew install node` or from [nodejs.org](https://nodejs.org))
- Docker & Docker Compose (`brew install docker`)

### 1. Build Rust Services

```bash
cd hft-demo
cargo build --release
```

### 2. Install Web UI Dependencies

```bash
cd web-ui
npm install
```

### 3. Run the Complete System

**Terminal 1: Market Simulator**
```bash
cargo run --release --bin market_simulator
```

**Terminal 2: Feed Handler**
```bash
cargo run --release --bin feed_handler
```

**Terminal 3: Strategy Engine**
```bash
cargo run --release --bin strategy_engine
```

**Terminal 4: Order Gateway**
```bash
cargo run --release --bin order_gateway
```

**Terminal 5: Telemetry Service**
```bash
cargo run --release --bin telemetry
```

**Terminal 6: Web Dashboard**
```bash
cd web-ui
npm run dev
```

**Terminal 7: Prometheus + Grafana (Optional)**
```bash
cd infra
docker-compose up -d
```

## 🎯 Access Points

- **Web Dashboard**: http://localhost:3000
- **Telemetry WebSocket**: ws://localhost:9090/ws
- **Prometheus Metrics**: http://localhost:9090/metrics
- **Prometheus UI** (Docker): http://localhost:9091
- **Grafana** (Docker): http://localhost:3001 (admin/admin)

## 📊 What You'll See

### Dashboard Metrics

1. **Ticks Received** - Total market ticks processed
2. **Orders Placed** - Total trading orders executed
3. **P50 Latency** - Median tick-to-strategy latency (µs)
4. **P99 Latency** - 99th percentile latency (µs)

### Live Charts

- **Latency Over Time** - Real-time line chart showing p50, p99, and mean latency
- **Latency Distribution** - Bar chart of current latency percentiles

### System Performance

- **Target**: < 100µs end-to-end latency
- **Throughput**: 10,000 ticks/second
- **Transport**: Lock-free crossbeam channels + UDP

## 🔧 Configuration

All system parameters are now centralized in `config.toml`:

```toml
[system]
tick_rate = 10000  # ticks per second

[symbols]
enabled = ["BTC/USD", "ETH/USD", "SOL/USD", "AVAX/USD"]

[symbols.thresholds]
"BTC/USD" = { low = 44000.0, high = 46000.0 }

[strategy]
type = "threshold"  # Options: threshold, market_making, mean_reversion
order_size = 1.0

[metrics]
prometheus_enabled = true
export_interval_ms = 1000
```

## 🧪 Testing & Benchmarking

### Run Performance Benchmarks

```bash
cargo bench
```

This runs Criterion benchmarks for:
- Tick serialization/deserialization
- Order creation latency
- Latency measurement overhead

### Run Integration Tests

```bash
cargo test
```

Tests include:
- Market tick validation
- Order book operations
- Strategy behavior
- Market replay functionality

### Latency Measurement

The system measures **three latency points**:

1. **Network Latency**: Time from tick generation to receipt (UDP)
2. **Processing Latency**: Time from receipt to strategy decision
3. **Order Latency**: Time from signal to order placement

All metrics are exposed via Prometheus histograms with µs precision.

### Market Replay for Backtesting

Record live market data:
```rust
use hft_types::replay::MarketRecorder;

let mut recorder = MarketRecorder::new("data/market_2024.jsonl")?;
recorder.record_tick(&tick)?;
```

Replay for backtesting:
```rust
use hft_types::replay::MarketReplayer;

let mut replayer = MarketReplayer::new("data/market_2024.jsonl")?;
while let Some(tick) = replayer.next_tick()? {
    // Process tick through strategy
}
```

## 📈 Prometheus Queries

Access Prometheus at http://localhost:9091 and try:

```promql
# Tick rate
rate(feed_ticks_received_total[1m])

# P99 latency
histogram_quantile(0.99, feed_latency_micros_bucket)

# Order rate
rate(gateway_orders_placed_total[1m])
```

## 🐳 Docker Deployment (Optional)

For a production-like setup:

1. Start infrastructure:
```bash
cd infra
docker-compose up -d
```

2. Build and run services as Docker containers (Dockerfile creation left as exercise)

## 🛠️ Development

### Adding New Metrics

1. Define in service (e.g., `feed_handler/src/main.rs`):
```rust
pub static ref MY_METRIC: IntCounter = IntCounter::new("my_metric", "Description").unwrap();
```

2. Register in `init_metrics()`:
```rust
REGISTRY.register(Box::new(MY_METRIC.clone())).unwrap();
```

3. Instrument your code:
```rust
MY_METRIC.inc();
```

### Adding New Symbols

Update `config.toml` with new symbols and thresholds:

```toml
[symbols]
enabled = ["BTC/USD", "ETH/USD", "NEW/SYMBOL"]

[symbols.thresholds]
"NEW/SYMBOL" = { low = 100.0, high = 200.0 }

[symbols.base_prices]
"NEW/SYMBOL" = 150.0
```

### Creating Custom Strategies

Implement the `Strategy` trait from `hft-types`:

```rust
use hft_types::strategies::Strategy;
use hft_types::{EnrichedTick, TradingSignal};

pub struct MyStrategy {
    // Your strategy state
}

impl Strategy for MyStrategy {
    fn process_tick(&mut self, tick: &EnrichedTick) -> Option<TradingSignal> {
        // Your strategy logic
    }

    fn name(&self) -> &str {
        "MyStrategy"
    }
}
```

## 📝 Architecture Decisions

- **Shared Types Library**: Centralized data structures prevent duplication and ensure consistency
- **UDP over TCP**: Prioritizes latency over reliability (acceptable for market data)
- **Crossbeam Channels**: Lock-free MPMC channels for inter-thread communication
- **IPC Framework**: TCP-based message framing for reliable inter-process communication
- **Strategy Pattern**: Pluggable strategies (Threshold, Market Making, Mean Reversion)
- **Prometheus**: Industry-standard metrics with histograms for latency percentiles
- **WebSocket**: Real-time push for dashboard updates without polling
- **Tokio**: Async runtime for efficient I/O multiplexing
- **Market Replay**: Record and replay functionality for strategy backtesting

## 🔍 Troubleshooting

**Issue**: Dashboard shows "Disconnected"
- Ensure telemetry service is running on port 9090
- Check WebSocket URL in `web-ui/src/app/page.tsx`

**Issue**: No ticks received
- Verify market_simulator and feed_handler are running
- Check UDP port 9001 is not blocked

**Issue**: High latency (> 1000µs)
- System under load - close other applications
- Reduce tick rate in market_simulator
- Check CPU affinity settings

## 📚 Recent Improvements (v0.2.0)

✅ **Completed:**
- Shared types library (`hft-types`) with centralized data structures
- IPC messaging framework with TCP framing
- Unified configuration system (`config.toml`)
- Market replay system for backtesting
- Advanced strategies (Market Making, Mean Reversion)
- Order book manager with L2 reconstruction
- Performance benchmarks using Criterion
- Comprehensive integration tests

## 📚 Future Enhancements

- [ ] Add real exchange connectors (Binance, Coinbase WebSocket)
- [ ] Real-time order book visualization in dashboard
- [ ] Multi-strategy portfolio with risk management
- [ ] GPU acceleration for signal processing
- [ ] Kernel bypass networking (DPDK)
- [ ] FPGA tick-to-trade pipeline
- [ ] Machine learning strategy using LSTM/Transformer

## 📄 License

MIT License - feel free to use for learning and production!

## 🤝 Contributing

This is a demo project, but PRs for improvements are welcome!

---

**Built with ⚡ by Rust & Next.js for ultra-low-latency trading**
