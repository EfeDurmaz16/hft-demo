use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hft_types::{MarketTick, OrderSide, Order};
use std::time::{SystemTime, UNIX_EPOCH};

fn bench_tick_serialization(c: &mut Criterion) {
    let tick = MarketTick::new(
        "BTC/USD".to_string(),
        45000.0,
        100,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
    );

    c.bench_function("tick_serialize", |b| {
        b.iter(|| {
            black_box(serde_json::to_vec(&tick).unwrap())
        })
    });
}

fn bench_tick_deserialization(c: &mut Criterion) {
    let tick = MarketTick::new(
        "BTC/USD".to_string(),
        45000.0,
        100,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
    );
    let data = serde_json::to_vec(&tick).unwrap();

    c.bench_function("tick_deserialize", |b| {
        b.iter(|| {
            black_box(serde_json::from_slice::<MarketTick>(&data).unwrap())
        })
    });
}

fn bench_order_creation(c: &mut Criterion) {
    c.bench_function("order_create", |b| {
        b.iter(|| {
            black_box(Order::new(
                1,
                "BTC/USD".to_string(),
                OrderSide::Buy,
                45000.0,
                1.0,
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            ))
        })
    });
}

fn bench_latency_measurement(c: &mut Criterion) {
    c.bench_function("latency_calc", |b| {
        b.iter(|| {
            let send_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let recv_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            black_box((recv_time - send_time) as f64 / 1000.0)
        })
    });
}

criterion_group!(
    benches,
    bench_tick_serialization,
    bench_tick_deserialization,
    bench_order_creation,
    bench_latency_measurement
);
criterion_main!(benches);
