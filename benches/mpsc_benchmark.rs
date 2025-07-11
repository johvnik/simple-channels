use criterion::{Bencher, Criterion, Throughput, criterion_group, criterion_main};
use simple_channels::mpsc;
use std::thread;

/// Measures the throughput of sending a batch of messages through the channel.
fn mpsc_throughput_benchmark(c: &mut Criterion) {
    const BATCH_SIZE: u64 = 1000;
    let mut group = c.benchmark_group("mpsc_throughput");
    group.throughput(Throughput::Elements(BATCH_SIZE));

    group.bench_function("cap-64", |b: &mut Bencher| {
        b.iter(|| {
            let (tx, rx) = mpsc::bounded(64);
            thread::scope(|s| {
                s.spawn(|| {
                    for i in 0..BATCH_SIZE {
                        tx.send(i).unwrap();
                    }
                });
                for _ in 0..BATCH_SIZE {
                    rx.recv().unwrap();
                }
            });
        });
    });

    group.finish();
}

/// Measures the round-trip latency of a single message.
fn mpsc_latency_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("mpsc_latency");

    group.bench_function("cap-1", |b: &mut Bencher| {
        b.iter(|| {
            let (ping_tx, ping_rx) = mpsc::bounded(1);
            let (pong_tx, pong_rx) = mpsc::bounded(1);

            thread::scope(|s| {
                // The "pong" thread echoes back any message it receives on the ping channel.
                s.spawn(|| {
                    let msg = ping_rx.recv().unwrap();
                    pong_tx.send(msg).unwrap();
                });

                // The main thread sends a "ping" and waits for the "pong".
                ping_tx.send(0u64).unwrap();
                let _ = pong_rx.recv().unwrap();
            });
        });
    });

    group.finish();
}

criterion_group!(benches, mpsc_throughput_benchmark, mpsc_latency_benchmark);
criterion_main!(benches);
