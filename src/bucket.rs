use crate::bloom::BloomFilter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Bucket {
    messages: Vec<usize>,
    bloom_filter: Vec<u128>,
    bloom_count: u8,
    bloom_size: usize,
    bloom_k: u64,
}

impl Bucket {
    pub fn new(
        bloom_size: usize,
        bloom_k: u64,
    ) -> Self {
        Self {
            messages: vec![0; 128],
            bloom_filter: vec![0; bloom_size * 128],
            bloom_count: 0,
            bloom_size,
            bloom_k,
        }
    }

    pub fn add_message(&mut self, trigrams: &[String], key:usize) {
        let mut bloom_filter = BloomFilter::new(self.bloom_size * 128, self.bloom_k );

        trigrams.iter().for_each(|v| {
            bloom_filter.add(v)
        });
        self.add_bloom(bloom_filter.get_bitset());
        self.messages[(self.bloom_count - 1) as usize] = key;
    }

    // add current document to the bloom index
    fn add_bloom(&mut self, vec: &[u128]) {
        for i in 0..self.bloom_size * 128 as usize {
            if vec[i / 128] & (1 << (i % 128)) != 0 {
                self.bloom_filter[i] |= 1u128 << (self.bloom_count);
            }
        }
        self.bloom_count += 1
    }

    pub fn is_full(&self) -> bool {
        self.bloom_count == 128
    }

    #[inline(always)]
    pub fn search(&self, query_bits: &[u128]) -> Vec<usize> {
        let mut results = Vec::new();
        let mut res: u128;

        for i in (0..self.bloom_filter.len()).step_by(self.bloom_size * 128) {
            res = self.bloom_filter[query_bits[0] as usize + i];

            for &query_bit in query_bits[1..].iter() {
                res = res & self.bloom_filter[query_bit as usize + i];
                if res == 0 {
                    break;
                }
            }

            if res != 0 {
                for j in 0..128 {
                    if res & (1 << j) > 0 {
                        results.push(((128 * (i as u64 / (self.bloom_size * 128) as u64)) + j as u64) as usize);
                    }
                }
            }
        }
        results
    }
}