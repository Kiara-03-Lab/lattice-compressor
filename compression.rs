//! Benchmarks for ILC compression

use criterion::{criterion_group, criterion_main, Criterion, black_box};
use ilc_rs::{RingElement, RingLWEKey, AlgebraicShield, N};
use rand::Rng;

fn create_test_key() -> (RingLWEKey, [u8; 32]) {
    let seed = rand::thread_rng().gen::<[u8; 32]>();
    let a = RingElement::from_seed(&seed, 0);
    let b = RingElement::from_seed(&seed, 1);
    (RingLWEKey { a, b }, seed)
}

fn bench_compression(c: &mut Criterion) {
    let (key, seed) = create_test_key();
    
    c.bench_function("compress", |b| {
        b.iter(|| {
            black_box(key.compress(seed))
        })
    });
}

fn bench_decompression(c: &mut Criterion) {
    let (key, seed) = create_test_key();
    let compressed = key.compress(seed);
    
    c.bench_function("decompress", |b| {
        b.iter(|| {
            black_box(RingLWEKey::decompress(&compressed).unwrap())
        })
    });
}

fn bench_roundtrip(c: &mut Criterion) {
    let (key, seed) = create_test_key();
    
    c.bench_function("roundtrip", |b| {
        b.iter(|| {
            let compressed = key.compress(seed);
            black_box(RingLWEKey::decompress(&compressed).unwrap())
        })
    });
}

fn bench_serialization(c: &mut Criterion) {
    let (key, seed) = create_test_key();
    let compressed = key.compress(seed);
    
    c.bench_function("serialize", |b| {
        b.iter(|| {
            black_box(ilc_rs::to_bytes(&compressed))
        })
    });
    
    let bytes = ilc_rs::to_bytes(&compressed);
    c.bench_function("deserialize", |b| {
        b.iter(|| {
            black_box(ilc_rs::from_bytes(&bytes).unwrap())
        })
    });
}

/// Simulate bandwidth savings on a 100kbps IoT link
fn bench_bandwidth_simulation(c: &mut Criterion) {
    let (key, seed) = create_test_key();
    let compressed = key.compress(seed);
    
    let original_bytes = key.size_bytes();
    let compressed_bytes = ilc_rs::to_bytes(&compressed).len();
    
    // 100 kbps = 12,500 bytes/sec
    let bps = 12_500.0;
    let original_time_ms = (original_bytes as f64 / bps) * 1000.0;
    let compressed_time_ms = (compressed_bytes as f64 / bps) * 1000.0;
    
    println!("\n=== IoT Bandwidth Simulation (100 kbps) ===");
    println!("Original key: {} bytes ({:.1} ms)", original_bytes, original_time_ms);
    println!("Compressed:   {} bytes ({:.1} ms)", compressed_bytes, compressed_time_ms);
    println!("Savings:      {:.1} ms ({:.1}% reduction)", 
             original_time_ms - compressed_time_ms,
             (1.0 - compressed_bytes as f64 / original_bytes as f64) * 100.0);
    
    // The actual benchmark measures if reconstruction CPU cost is worth the bandwidth savings
    c.bench_function("reconstruct_vs_transfer", |b| {
        b.iter(|| {
            black_box(RingLWEKey::decompress(&compressed).unwrap())
        })
    });
}

criterion_group!(
    benches,
    bench_compression,
    bench_decompression,
    bench_roundtrip,
    bench_serialization,
    bench_bandwidth_simulation,
);
criterion_main!(benches);
