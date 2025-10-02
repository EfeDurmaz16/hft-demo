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

1. **market_simulator** - Generates fake market ticks over UDP at 10k/sec
2. **feed_handler** - Receives UDP ticks, measures latency, forwards to strategy
3. **strategy_engine** - Simple threshold-based trading strategy
4. **order_gateway** - Simulates order placement with latency tracking
5. **telemetry** - Prometheus metrics + WebSocket feed for dashboard

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

### Market Simulator

Edit `market_simulator/src/main.rs`:

```rust
let ticks_per_second = 10_000; // Adjust tick rate
let symbols = vec!["BTC/USD", "ETH/USD", ...]; // Add symbols
```

### Strategy Engine

Edit `strategy_engine/src/main.rs`:

```rust
thresholds.insert("BTC/USD".to_string(), (44000.0, 46000.0)); // (low, high)
```

### Telemetry Update Rate

Edit `telemetry/src/main.rs`:

```rust
let mut interval = tokio::time::interval(Duration::from_millis(500)); // WebSocket update rate
```

## 🧪 Testing Latency

The system measures **three latency points**:

1. **Network Latency**: Time from tick generation to receipt (UDP)
2. **Processing Latency**: Time from receipt to strategy decision
3. **Order Latency**: Time from signal to order placement

All metrics are exposed via Prometheus histograms with µs precision.

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

Edit the symbols list in `market_simulator/src/main.rs` and corresponding thresholds in `strategy_engine/src/main.rs`.

## 📝 Architecture Decisions

- **UDP over TCP**: Prioritizes latency over reliability (acceptable for market data)
- **Crossbeam Channels**: Lock-free MPMC channels for inter-thread communication
- **Prometheus**: Industry-standard metrics with histograms for latency percentiles
- **WebSocket**: Real-time push for dashboard updates without polling
- **Tokio**: Async runtime for efficient I/O multiplexing

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

## 📚 Further Enhancements

- [ ] Add real exchange connectors (Binance, Coinbase)
- [ ] Implement order book reconstruction
- [ ] Add market making strategy
- [ ] GPU acceleration for signal processing
- [ ] Kernel bypass networking (DPDK)
- [ ] FPGA tick-to-trade pipeline

## 📄 License

MIT License - feel free to use for learning and production!

## 🤝 Contributing

This is a demo project, but PRs for improvements are welcome!

---

**Built with ⚡ by Rust & Next.js for ultra-low-latency trading**
