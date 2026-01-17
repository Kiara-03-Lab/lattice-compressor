//! Ring arithmetic for Z_q[X]/(X^n + 1)
//! 
//! MVP: Uses naive polynomial multiplication. 
//! Production: Replace with NTT for O(n log n) performance.

/// Ring parameters (Kyber-512 compatible)
pub const N: usize = 256;
pub const Q: u32 = 3329;

/// Polynomial in Z_q[X]/(X^n + 1)
#[derive(Clone, Debug, PartialEq)]
pub struct RingElement {
    pub coeffs: [u16; N],
}

impl Default for RingElement {
    fn default() -> Self {
        Self { coeffs: [0u16; N] }
    }
}

impl RingElement {
    pub fn new(coeffs: [u16; N]) -> Self {
        Self { coeffs }
    }

    /// Reduce all coefficients mod q
    pub fn reduce(&mut self) {
        for c in &mut self.coeffs {
            *c = (*c as u32 % Q) as u16;
        }
    }

    /// Add two ring elements
    pub fn add(&self, other: &Self) -> Self {
        let mut result = Self::default();
        for i in 0..N {
            result.coeffs[i] = ((self.coeffs[i] as u32 + other.coeffs[i] as u32) % Q) as u16;
        }
        result
    }

    /// Subtract two ring elements
    pub fn sub(&self, other: &Self) -> Self {
        let mut result = Self::default();
        for i in 0..N {
            result.coeffs[i] = ((self.coeffs[i] as u32 + Q - other.coeffs[i] as u32) % Q) as u16;
        }
        result
    }

    /// Naive polynomial multiplication in Z_q[X]/(X^n + 1)
    /// MVP implementation - O(n^2). Replace with NTT for production.
    pub fn mul(&self, other: &Self) -> Self {
        let mut result = [0i64; 2 * N];
        
        // Standard polynomial multiplication
        for i in 0..N {
            for j in 0..N {
                result[i + j] += (self.coeffs[i] as i64) * (other.coeffs[j] as i64);
            }
        }
        
        // Reduce by X^n + 1 (coefficients at index >= N wrap with negation)
        let mut out = Self::default();
        for i in 0..N {
            let val = result[i] - result[i + N];
            // Handle negative values
            out.coeffs[i] = ((val % Q as i64 + Q as i64) % Q as i64) as u16;
        }
        out
    }

    /// Generate deterministically from seed
    pub fn from_seed(seed: &[u8; 32], domain: u8) -> Self {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;
        use sha3::{Shake128, digest::{ExtendableOutput, Update, XofReader}};
        
        // Domain-separate the seed
        let mut hasher = Shake128::default();
        hasher.update(seed);
        hasher.update(&[domain]);
        let mut reader = hasher.finalize_xof();
        
        let mut derived_seed = [0u8; 32];
        reader.read(&mut derived_seed);
        
        let mut rng = ChaCha20Rng::from_seed(derived_seed);
        let mut coeffs = [0u16; N];
        
        use rand::Rng;
        for c in &mut coeffs {
            *c = (rng.gen::<u16>() as u32 % Q) as u16;
        }
        
        Self { coeffs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_sub_inverse() {
        let a = RingElement::from_seed(&[1u8; 32], 0);
        let b = RingElement::from_seed(&[2u8; 32], 0);
        let sum = a.add(&b);
        let recovered = sum.sub(&b);
        assert_eq!(a, recovered);
    }

    #[test]
    fn test_mul_identity() {
        let a = RingElement::from_seed(&[1u8; 32], 0);
        let mut one = RingElement::default();
        one.coeffs[0] = 1;
        let result = a.mul(&one);
        assert_eq!(a, result);
    }
}
