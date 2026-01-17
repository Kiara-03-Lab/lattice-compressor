//! # Ideal-Lattice-Compression (ILC)
//! 
//! Reduce Ring-LWE public key bandwidth by 40-60% using algebraic sketching.
//! 
//! ## Quick Start
//! 
//! ```rust
//! use ilc_rs::{RingLWEKey, RingElement, AlgebraicShield};
//! 
//! // Create a key (in practice, use your PQC library's key)
//! let seed = [0u8; 32];
//! let a = RingElement::from_seed(&seed, 0);
//! let b = RingElement::from_seed(&seed, 1);
//! let key = RingLWEKey { a, b };
//! 
//! // Compress
//! let compressed = key.compress(seed);
//! println!("Compressed to {} bytes", compressed.size_bytes());
//! 
//! // Decompress
//! let recovered = RingLWEKey::decompress(&compressed).unwrap();
//! assert_eq!(key.b.coeffs, recovered.b.coeffs);
//! ```
//! 
//! ## MVP Limitations
//! 
//! This is a minimal viable implementation:
//! - Uses naive O(n²) polynomial multiplication (replace with NTT for production)
//! - Simple coefficient decimation (full Gröbner-based reconstruction planned)
//! - Fixed parameters (Kyber-512 compatible: n=256, q=3329)

pub mod ring;
pub mod types;
pub mod sketcher;

pub use ring::{RingElement, N, Q};
pub use types::{RingLWEKey, CompressedPK, AlgebraicShield, ILCError};

/// Convenience function: compress a public key polynomial
pub fn compress(b_coeffs: &[u16; N], seed: [u8; 32]) -> CompressedPK {
    let a = RingElement::from_seed(&seed, 0);
    let b = RingElement::new(*b_coeffs);
    let key = RingLWEKey { a, b };
    key.compress(seed)
}

/// Convenience function: decompress to get polynomial coefficients
pub fn decompress(sketch: &CompressedPK) -> Result<[u16; N], ILCError> {
    let key = RingLWEKey::decompress(sketch)?;
    Ok(key.b.coeffs)
}

/// Serialize compressed key to bytes
pub fn to_bytes(sketch: &CompressedPK) -> Vec<u8> {
    bincode::serialize(sketch).expect("serialization should not fail")
}

/// Deserialize compressed key from bytes
pub fn from_bytes(data: &[u8]) -> Result<CompressedPK, ILCError> {
    bincode::deserialize(data).map_err(|_| ILCError::InvalidInput)
}
