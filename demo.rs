//! Example: Basic compression/decompression workflow

use ilc_rs::{RingElement, RingLWEKey, AlgebraicShield, N};

fn main() {
    println!("=== ILC-RS: Ideal Lattice Compression Demo ===\n");
    
    // Simulate creating an RLWE key pair
    // In practice, you'd get this from your PQC library (e.g., pqcrypto-kyber)
    let seed: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];
    
    // Generate deterministic polynomials (simulating RLWE key generation)
    let a = RingElement::from_seed(&seed, 0);  // Public matrix element
    let s = RingElement::from_seed(&seed, 1);  // Secret key
    let e = RingElement::from_seed(&seed, 2);  // Error term
    
    // b = a*s + e (standard RLWE)
    let b = a.mul(&s).add(&e);
    
    let public_key = RingLWEKey { a: a.clone(), b };
    
    // Original size
    let original_size = public_key.size_bytes();
    println!("Original public key size: {} bytes", original_size);
    
    // Compress
    let compressed = public_key.compress(seed);
    let compressed_size = compressed.size_bytes();
    println!("Compressed size:          {} bytes", compressed_size);
    
    // Serialized wire format
    let wire_bytes = ilc_rs::to_bytes(&compressed);
    println!("Wire format size:         {} bytes", wire_bytes.len());
    
    // Compression stats
    let ratio = wire_bytes.len() as f64 / original_size as f64;
    println!("\nCompression ratio: {:.1}%", ratio * 100.0);
    println!("Space saved:       {:.1}%", (1.0 - ratio) * 100.0);
    
    // Decompress and verify
    println!("\n--- Decompression ---");
    let recovered = RingLWEKey::decompress(&compressed).expect("decompression failed");
    
    // Verify correctness
    let matches = public_key.b.coeffs == recovered.b.coeffs;
    println!("Coefficients match: {}", matches);
    
    if matches {
        println!("\n✓ Round-trip successful!");
    } else {
        println!("\n✗ ERROR: Reconstruction failed!");
        std::process::exit(1);
    }
    
    // IoT bandwidth calculation
    println!("\n--- IoT Bandwidth Impact (100 kbps link) ---");
    let bps = 12_500.0; // 100 kbps = 12,500 bytes/sec
    let original_ms = (original_size as f64 / bps) * 1000.0;
    let compressed_ms = (wire_bytes.len() as f64 / bps) * 1000.0;
    println!("Original transfer time:   {:.1} ms", original_ms);
    println!("Compressed transfer time: {:.1} ms", compressed_ms);
    println!("Time saved:               {:.1} ms", original_ms - compressed_ms);
}
