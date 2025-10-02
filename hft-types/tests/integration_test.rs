use hft_types::{MarketTick, Order, OrderSide, OrderBook, BookLevel};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_market_tick_creation() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let tick = MarketTick::new("BTC/USD".to_string(), 45000.0, 100, timestamp);

    assert_eq!(tick.symbol, "BTC/USD");
    assert_eq!(tick.price, 45000.0);
    assert_eq!(tick.volume, 100);
    assert_eq!(tick.timestamp_nanos, timestamp);
}

#[test]
fn test_order_serialization() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let order = Order::new(
        1,
        "ETH/USD".to_string(),
        OrderSide::Buy,
        2500.0,
        1.0,
        timestamp,
    );

    let serialized = serde_json::to_string(&order).unwrap();
    let deserialized: Order = serde_json::from_str(&serialized).unwrap();

    assert_eq!(order.order_id, deserialized.order_id);
    assert_eq!(order.symbol, deserialized.symbol);
    assert_eq!(order.price, deserialized.price);
}

#[test]
fn test_order_book_operations() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut book = OrderBook::new("BTC/USD".to_string(), timestamp);

    // Add bids (sorted highest to lowest)
    book.bids.push(BookLevel { price: 44900.0, quantity: 1.0 });
    book.bids.push(BookLevel { price: 44800.0, quantity: 2.0 });

    // Add asks (sorted lowest to highest)
    book.asks.push(BookLevel { price: 45100.0, quantity: 1.5 });
    book.asks.push(BookLevel { price: 45200.0, quantity: 3.0 });

    assert_eq!(book.best_bid().unwrap().price, 44900.0);
    assert_eq!(book.best_ask().unwrap().price, 45100.0);
    assert_eq!(book.spread().unwrap(), 200.0);
    assert_eq!(book.mid_price().unwrap(), 45000.0);
}

#[test]
fn test_tick_latency_calculation() {
    let send_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::thread::sleep(std::time::Duration::from_micros(100));

    let recv_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let latency_micros = (recv_time - send_time) as f64 / 1000.0;

    assert!(latency_micros >= 100.0);
    assert!(latency_micros < 1000.0); // Should be well under 1ms
}

#[test]
fn test_order_side_display() {
    assert_eq!(format!("{}", OrderSide::Buy), "BUY");
    assert_eq!(format!("{}", OrderSide::Sell), "SELL");
}
