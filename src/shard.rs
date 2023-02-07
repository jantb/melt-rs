use serde::{Deserialize, Serialize};

use crate::bloom::BloomFilter;
use crate::bucket::Bucket;

#[derive(Serialize, Deserialize)]
pub struct Shard {
    bucket: Vec<Bucket>,
    bloom_size: usize,
    bloom_k: u64,
}

impl Shard {
    pub fn new(
        bloom_size: usize,
        bloom_k: u64,
    ) -> Self {
        Self {
            bucket: vec![],
            bloom_size,
            bloom_k,
        }
    }

    pub fn get_m(&self) -> usize {
        return self.bloom_size;
    }
    pub fn get_k(&self) -> u64 {
        return self.bloom_k;
    }

    pub fn add_message(&mut self, trigrams: &[String], key:usize)  {
        self.get_bucket().add_message( trigrams,key)
    }

    #[inline(always)]
    pub fn search(&self, trigrams: &[String]) -> Vec<usize> {
        let query_bits = self.get_query_bits(trigrams);
        return self.bucket.iter().map(|b| b.search(&query_bits)).flatten().collect();
    }

    #[inline(always)]
    pub fn search_or(&self, trigrams: &[String]) -> Vec<usize> {
        let query_bits = self.get_query_bits(trigrams);
        return self.bucket.iter().map(|b| b.search_or(&query_bits)).flatten().collect();
    }

    #[inline(always)]
    fn get_query_bits(&self, trigrams: &[String]) -> Vec<u128> {
        let mut bloom_filter = BloomFilter::new(self.bloom_size * 128, self.bloom_k);
        trigrams.iter().for_each(|t| bloom_filter.add(t));
        return Self::get_set_bits(bloom_filter.get_bitset());
    }

    #[inline(always)]
    fn get_bucket(&mut self) -> &mut Bucket {
        if self.bucket.is_empty() || self.bucket.last().unwrap().is_full() {
            self.bucket.push(Bucket::new(self.bloom_size, self.bloom_k));
        }
        self.bucket.last_mut().unwrap()
    }

    #[inline(always)]
    fn get_set_bits(bits: &Vec<u128>) -> Vec<u128> {
        let mut set_bits = Vec::new();
        for (i, &bit) in bits.iter().enumerate() {
            for j in 0..128 {
                if bit & (1 << j) != 0 {
                    set_bits.push((i as u128) * 128 + j as u128);
                }
            }
        }
        set_bits
    }
}
