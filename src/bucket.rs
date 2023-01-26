use std::sync::atomic::Ordering;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use crate::bloom::BloomFilter;
use serde::{Deserialize, Serialize};
use crate::index::GLOBAL_COUNT;

#[derive(Serialize, Deserialize)]
pub struct Bucket {
    messages: Vec<usize>,
    bloom_filter: Vec<u64>,
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
            messages: vec![0; 64],
            bloom_filter: vec![0; bloom_size * 64],
            bloom_count: 0,
            bloom_size,
            bloom_k,
        }
    }

    pub fn add_message(&mut self, message: &str, trigrams: &Vec<String>, conn: &DBWithThreadMode<MultiThreaded>) {
        let mut bloom_filter = BloomFilter::new(self.bloom_size * 64, self.bloom_k);

        trigrams.iter().for_each(|v| {
            bloom_filter.add(v)
        });
        self.add_bloom(bloom_filter.get_bitset());
        let count = GLOBAL_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
        conn.put(count.to_le_bytes(), message).unwrap();
        self.messages[(self.bloom_count - 1) as usize] = count;
    }

    // add current document to the bloom index
    fn add_bloom(&mut self, vec: &Vec<u64>) {
        for i in 0..self.bloom_size * 64 as usize {
            if vec[i / 64] & (1 << (i % 64)) != 0 {
                self.bloom_filter[i] |= 1u64 << (self.bloom_count);
            }
        }
        self.bloom_count += 1
    }

    pub fn is_full(&self) -> bool {
        self.bloom_count == 64
    }

    pub fn search(&self, query: &str, query_bits: &Vec<u64>, conn: &DBWithThreadMode<MultiThreaded>) -> Vec<String> {
        let mut results = Vec::new();
        let mut res: u64;

        for i in (0..self.bloom_filter.len()).step_by(self.bloom_size * 64) {
            res = self.bloom_filter[query_bits[0] as usize + i];

            for &query_bit in query_bits[1..].iter() {
                res = res & self.bloom_filter[query_bit as usize + i];
                if res == 0 {
                    break;
                }
            }

            if res != 0 {
                for j in 0..64 {
                    if res & (1 << j) > 0 {
                        results.push((64 * (i as u64 / (self.bloom_size * 64) as u64)) + j as u64);
                    }
                }
            }
        }
        if results.is_empty() { return vec![]; }
        let vec: Vec<_> = results.iter().map(|i| self.messages[*i as usize]).collect();
        let mut messages = Vec::new();
        for x in vec {
            let result = String::from_utf8(conn.get(x.to_le_bytes()).unwrap().unwrap()).unwrap();
            if result.contains(&query) {
                messages.push(result);
            }
        }

        messages
    }
}