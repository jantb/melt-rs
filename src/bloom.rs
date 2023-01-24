use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct BloomFilter {
    bitset: Vec<u64>,
    num_bits: usize,
    hashes: u64,
}

impl BloomFilter {
    pub fn new(num_bits: usize, hashes: u64) -> Self {
        let num_words = (num_bits + 63) / 64;
        Self {
            bitset: vec![0; num_words],
            num_bits,
            hashes,
        }
    }

    pub fn add<T: Hash>(&mut self, elem: &T) {
        for i in 0..self.hashes {
            let mut hasher = DefaultHasher::new();
            elem.hash(&mut hasher);
            hasher.write_usize(i as usize);
            let hash = hasher.finish();

            let idx = (hash as usize) % self.num_bits;
            let word_idx = idx / 64;
            let bit_idx = idx % 64;
            self.bitset[word_idx] |= 1 << bit_idx;
        }
    }

    pub fn get_bitset(&self) -> &Vec<u64> {
        return &self.bitset;
    }
}

pub fn estimate_parameters(n: u64, p: f64) -> (usize, u64) {
    let m = ((-1.0 * n as f64 * p.ln()) / (2.0_f64.ln().powi(2))).ceil() as u64;
    let k = ((2.0_f64.ln() * m as f64) / n as f64).ceil() as u64;
    let m = 64 * ((m + 63) / 64) / 64;
    (m as usize, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let num_bits = 100;
        let hashes = 5;
        let bf = BloomFilter::new(num_bits, hashes);
        assert_eq!(bf.num_bits, num_bits);
        assert_eq!(bf.hashes, hashes);
        assert_eq!(bf.bitset.len(), (num_bits + 63) / 64);
    }

    #[test]
    fn test_add() {
        let mut bf = BloomFilter::new(100, 5);
        bf.add(&"hello");
        let bitset = bf.get_bitset();
        assert_ne!(bitset[0], 0);
    }
}