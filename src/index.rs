use std::fs;
use std::fs::File;
use std::io::{Error, Read};
use std::sync::atomic::AtomicUsize;
use serde::{Deserialize, Serialize};
use crate::bloom::estimate_parameters;

use crate::shard::Shard;
use crate::trigrams::trigram;

pub static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Serialize, Deserialize, Default)]
pub struct SearchIndex {
    shards: Vec<Shard>,
    size: usize,
}

impl SearchIndex {
    pub fn default() -> SearchIndex {
        SearchIndex{ shards: vec![], size: 0 }
    }
    pub fn clear(&mut self) {
        self.size = 0;
        self.shards.clear();
    }

    pub fn add_message(&mut self, message: &str, key:usize) {
        self.size += 1;
        let trigrams = trigram(message.to_lowercase().as_str());
        let (m, k) = estimate_parameters(trigrams.len() as u64, 0.6);
        match self.shards.iter_mut().find(|s| s.get_m() == m && s.get_k() == k) {
            None => {
                let mut shard = Shard::new(m, k);
                let i = shard.add_message(&trigrams, key);
                self.shards.push(shard);
                i
            }
            Some(shard) => { shard.add_message(&trigrams, key) }
        };
    }

    pub fn search(&self, query: &str) -> Vec<usize> {
        if query.len() < 3 { return vec![]; }
        let results: Vec<_> = self.shards
            .iter()
            .flat_map(|shard| shard.search(query.to_lowercase().as_str()))
            .collect();
        results
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_size_bytes(&self) -> usize {
        let serialized: Vec<u8> = bincode::serialize(&self.shards).unwrap();
        serialized.len()
    }
}

fn get_file_as_byte_vec(filename: &String) -> Result<Vec<u8>, Error> {
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;

    Ok(buffer)
}


