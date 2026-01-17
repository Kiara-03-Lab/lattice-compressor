//! Core data structures for ILC

use serde::{Serialize, Deserialize};
use crate::ring::{RingElement, N, Q};

/// Standard RLWE public key: pk = (a, b) where b = a*s + e
#[derive(Clone, Debug)]
pub struct RingLWEKey {
    pub a: RingElement,
    pub b: RingElement,
}

impl RingLWEKey {
    /// Size in bytes of uncompressed key
    pub fn size_bytes(&self) -> usize {
        // 2 polynomials * N coefficients * 2 bytes each (for q < 2^16)
        2 * N * 2
    }
}

/// Compressed public key using algebraic sketching
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompressedPK {
    /// Seed to regenerate polynomial 'a'
    pub seed: [u8; 32],
    
    /// Anchor coefficients (every 2nd coefficient of b)
    pub anchor_coeffs: Vec<u16>,
    
    /// Checksum for verification (hash of original b)
    pub checksum: [u8; 8],
    
    /// Parity coefficients for reconstruction
    /// Stores XOR-like algebraic checksums for recovery
    pub parity: Vec<u16>,
}

impl CompressedPK {
    /// Size in bytes of compressed key
    pub fn size_bytes(&self) -> usize {
        32 + // seed
        self.anchor_coeffs.len() * 2 +
        8 + // checksum
        self.parity.len() * 2
    }
}

/// Error type for compression/decompression
#[derive(Debug, Clone)]
pub enum ILCError {
    ReconstructionFailed,
    ChecksumMismatch,
    InvalidInput,
}

impl std::fmt::Display for ILCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ILCError::ReconstructionFailed => write!(f, "Failed to reconstruct key"),
            ILCError::ChecksumMismatch => write!(f, "Checksum verification failed"),
            ILCError::InvalidInput => write!(f, "Invalid input data"),
        }
    }
}

impl std::error::Error for ILCError {}

/// Trait for algebraic compression
pub trait AlgebraicShield {
    fn compress(&self, seed: [u8; 32]) -> CompressedPK;
    fn decompress(sketch: &CompressedPK) -> Result<Self, ILCError> where Self: Sized;
}
