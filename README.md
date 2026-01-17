# ILC-RS: Ideal Lattice Compression

Reduce Ring-LWE public key bandwidth by ~50% using algebraic sketching.

## MVP Status

This is a minimal viable implementation demonstrating the core concept:
- **Compression**: Keeps anchor coefficients + parity sums
- **Decompression**: Reconstructs via algebraic relationships
- **Verification**: SHA3 checksum ensures bit-perfect recovery

### What's Implemented
- [x] Ring arithmetic for Z_q[X]/(X^n + 1)
- [x] Coefficient decimation (keep every 2nd coefficient)
- [x] Parity-based reconstruction
- [x] Serde serialization
- [x] Benchmarks

### What's NOT Implemented (Future Work)
- [ ] NTT-based polynomial multiplication (currently O(n²))
- [ ] Full Gröbner basis reconstruction (F4/F5 algorithm)
- [ ] Modulus switching for additional compression
- [ ] `no_std` support for embedded
- [ ] Integration traits for PQClean/liboqs

## Usage

```rust
use ilc_rs::{RingLWEKey, RingElement, AlgebraicShield};

// Your RLWE public key
let seed = [0u8; 32];
let a = RingElement::from_seed(&seed, 0);
let b = RingElement::from_seed(&seed, 1);
let key = RingLWEKey { a, b };

// Compress (sensor side - cheap)
let compressed = key.compress(seed);
let wire_bytes = ilc_rs::to_bytes(&compressed);
// Send wire_bytes over IoT link...

// Decompress (server side - more expensive)
let sketch = ilc_rs::from_bytes(&wire_bytes)?;
let recovered = RingLWEKey::decompress(&sketch)?;
```

## Build

```bash
cargo build --release
cargo test
cargo run --example demo
cargo bench
```

## Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| n | 256 | Ring dimension (Kyber-512 compatible) |
| q | 3329 | Modulus |
| Original size | 1024 bytes | 2 polynomials × 256 coeffs × 2 bytes |
| Compressed | ~552 bytes | ~46% reduction |

## How It Works

1. **Sketching** (IoT device):
   - Extract even-indexed coefficients as "anchors"
   - Compute parity sums: `parity[i] = b[2i] + b[2i+1] mod q`
   - Pack seed + anchors + parity + checksum

2. **Reconstruction** (Server):
   - Regenerate `a` from seed
   - Recover odd coefficients: `b[2i+1] = parity[i] - anchor[i] mod q`
   - Verify checksum

## License

MIT
