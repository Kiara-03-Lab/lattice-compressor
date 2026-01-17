//! Sketcher: Compression engine using coefficient decimation
//!
//! MVP Strategy:
//! 1. Keep every 2nd coefficient (anchors)
//! 2. Store parity sums for adjacent pairs to enable reconstruction
//! 3. Include checksum for verification

use crate::ring::{RingElement, N, Q};
use crate::types::{RingLWEKey, CompressedPK, AlgebraicShield, ILCError};
use sha3::{Sha3_256, Digest};

/// Compute checksum of polynomial coefficients
fn compute_checksum(poly: &RingElement) -> [u8; 8] {
    let mut hasher = Sha3_256::new();
    for c in &poly.coeffs {
        hasher.update(c.to_le_bytes());
    }
    let hash = hasher.finalize();
    let mut checksum = [0u8; 8];
    checksum.copy_from_slice(&hash[..8]);
    checksum
}

impl AlgebraicShield for RingLWEKey {
    /// Compress the public key using algebraic sketching
    fn compress(&self, seed: [u8; 32]) -> CompressedPK {
        // Extract anchor coefficients (even indices)
        let anchor_coeffs: Vec<u16> = self.b.coeffs
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, &c)| c)
            .collect();
        
        // Compute parity: sum of adjacent pairs mod q
        // This allows reconstruction: if we know anchor[i] and parity[i],
        // we can recover odd[i] = parity[i] - anchor[i] mod q
        let parity: Vec<u16> = (0..N/2)
            .map(|i| {
                let even = self.b.coeffs[2*i] as u32;
                let odd = self.b.coeffs[2*i + 1] as u32;
                ((even + odd) % Q) as u16
            })
            .collect();
        
        let checksum = compute_checksum(&self.b);
        
        CompressedPK {
            seed,
            anchor_coeffs,
            checksum,
            parity,
        }
    }
    
    /// Decompress and reconstruct the public key
    fn decompress(sketch: &CompressedPK) -> Result<Self, ILCError> {
        // Regenerate 'a' from seed
        let a = RingElement::from_seed(&sketch.seed, 0);
        
        // Reconstruct 'b' from anchors and parity
        let mut b = RingElement::default();
        
        if sketch.anchor_coeffs.len() != N/2 || sketch.parity.len() != N/2 {
            return Err(ILCError::InvalidInput);
        }
        
        for i in 0..N/2 {
            let anchor = sketch.anchor_coeffs[i] as u32;
            let parity = sketch.parity[i] as u32;
            
            // Even coefficient is the anchor
            b.coeffs[2*i] = anchor as u16;
            
            // Odd coefficient: parity - anchor mod q
            b.coeffs[2*i + 1] = ((parity + Q - anchor) % Q) as u16;
        }
        
        // Verify checksum
        let computed_checksum = compute_checksum(&b);
        if computed_checksum != sketch.checksum {
            return Err(ILCError::ChecksumMismatch);
        }
        
        Ok(RingLWEKey { a, b })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn random_key() -> (RingLWEKey, [u8; 32]) {
        let seed = rand::thread_rng().gen::<[u8; 32]>();
        let a = RingElement::from_seed(&seed, 0);
        let s = RingElement::from_seed(&seed, 1); // secret
        let e = RingElement::from_seed(&seed, 2); // error (small in practice)
        let b = a.mul(&s).add(&e);
        
        (RingLWEKey { a, b }, seed)
    }

    #[test]
    fn test_compress_decompress_roundtrip() {
        let (key, seed) = random_key();
        let compressed = key.compress(seed);
        let recovered = RingLWEKey::decompress(&compressed).unwrap();
        
        assert_eq!(key.b.coeffs, recovered.b.coeffs);
    }

    #[test]
    fn test_compression_ratio() {
        let (key, seed) = random_key();
        let compressed = key.compress(seed);
        
        let original_size = key.size_bytes();
        let compressed_size = compressed.size_bytes();
        let ratio = compressed_size as f64 / original_size as f64;
        
        println!("Original: {} bytes", original_size);
        println!("Compressed: {} bytes", compressed_size);
        println!("Ratio: {:.2}%", ratio * 100.0);
        
        // Should achieve ~50% compression with this MVP approach
        assert!(ratio < 0.75, "Compression ratio should be under 75%");
    }
}
