use rusqlite::Connection;

use crate::bloom::BloomFilter;
use crate::bucket::Bucket;
use crate::message::Message;
use crate::trigrams::trigram;

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

    pub fn add_message(&mut self, message:  &Message, trigrams: &Vec<String>, conn: &Connection) {
        self.get_bucket().add_message(message, trigrams, conn)
    }

    pub fn search(&self, query: &str, conn: &Connection) -> Vec<Message> {
        let query_bits = self.get_query_bits(trigram(&*query.to_lowercase()));
        return self.bucket.iter().map(|b| b.search(query, &query_bits, conn)).flatten().collect()
    }

    fn get_query_bits(&self, trigrams: Vec<String>) -> Vec<u64> {
        let mut bloom_filter = BloomFilter::new(self.bloom_size*64, self.bloom_k);
        trigrams.iter().for_each(|t| bloom_filter.add(t));
        return Self::get_set_bits(bloom_filter.get_bitset());
    }

    fn get_bucket(&mut self) ->  &mut Bucket {
        if self.bucket.is_empty() || self.bucket.last().unwrap().is_full() {
            self.bucket.push(Bucket::new(self.bloom_size, self.bloom_k));
        }
        self.bucket.last_mut().unwrap()
    }

    fn get_set_bits(bits: &Vec<u64>) -> Vec<u64> {
        let mut set_bits = Vec::new();
        for (i, &bit) in bits.iter().enumerate() {
            for j in 0..64 {
                if bit & (1 << j) != 0 {
                    set_bits.push((i as u64) * 64 + j as u64);
                }
            }
        }
        set_bits
    }
}
