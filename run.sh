#!/bin/bash

# HFT Demo - Quick Start Script
# This script runs all services in separate terminal windows (macOS)

echo "🚀 Starting HFT Demo System..."

# Function to run command in new terminal
run_in_terminal() {
    osascript -e "tell application \"Terminal\" to do script \"cd $(pwd) && $1\""
}

# Build all Rust services
echo "📦 Building Rust services..."
cargo build --release

# Start services in separate terminals
echo "🔄 Launching services..."

run_in_terminal "cargo run --release --bin market_simulator"
sleep 1

run_in_terminal "cargo run --release --bin feed_handler"
sleep 1

run_in_terminal "cargo run --release --bin strategy_engine"
sleep 1

run_in_terminal "cargo run --release --bin order_gateway"
sleep 1

run_in_terminal "cargo run --release --bin telemetry"
sleep 2

# Start web UI
run_in_terminal "cd web-ui && npm install && npm run dev"

echo "✅ All services launched!"
echo ""
echo "📊 Access Points:"
echo "  - Web Dashboard: http://localhost:3000"
echo "  - Telemetry:     http://localhost:9090/metrics"
echo ""
echo "To start Prometheus & Grafana:"
echo "  cd infra && docker-compose up -d"
echo "  - Prometheus:    http://localhost:9091"
echo "  - Grafana:       http://localhost:3001 (admin/admin)"
