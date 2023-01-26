use std::fs;
use std::fs::File;
use std::io::{Error, Read};
use std::sync::atomic::AtomicUsize;
use bincode::deserialize;
use rayon::prelude::*;
use rocksdb::{DB, DBCompressionType, DBWithThreadMode, MultiThreaded, Options};
use crate::bloom::estimate_parameters;

use crate::shard::Shard;
use crate::trigrams::trigram;

pub static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct SearchIndex {
    shards: Vec<Shard>,
    thread: usize,
    conn: DBWithThreadMode<MultiThreaded>,
}

fn default_conn(thread: u8) -> DBWithThreadMode<MultiThreaded> {
    let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
    let path = format!("{}/.melt{thread}.db", buf);
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_compression_type(DBCompressionType::Zstd);
    opts.set_zstd_max_train_bytes(100 * 16384);
    opts.optimize_for_point_lookup(10);
    let db: DBWithThreadMode<MultiThreaded> = DBWithThreadMode::open_cf(&opts, path, &["default"]).unwrap();
    db
}

impl SearchIndex {
    pub fn save_to_json(&self) -> Result<(), Error> {
        let serialized: Vec<u8> = bincode::serialize(&self.shards).unwrap();
        let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
        let path = format!("{}/.melt_index{}.dat", buf, self.thread);

        fs::write(path, serialized)?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.shards.clear();
        let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
        let path = format!("{}/.melt_index{}.dat", buf, self.thread);
        let _ = DB::destroy(&Options::default(), path);
    }

    pub fn load_from_json(thread: u8) -> Self {
        let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
        let path = format!("{}/.melt_index{}.dat", buf, thread);
        let file = get_file_as_byte_vec(&path);

        match file {
            Ok(file) => {
                let shards: Vec<Shard> = match deserialize(&file) {
                    Ok(shards) => shards,
                    Err(_) => vec![],
                };
                Self {
                    shards,
                    conn: default_conn(thread),
                    thread: thread as usize,
                }
            }
            Err(_) => {
                let index = Self {
                    shards: vec![],
                    conn: default_conn(thread),
                    thread: thread as usize,
                };
                index
            }
        }
    }

    pub fn add_message(&mut self, message: &str) {
        let trigrams = trigram(message);
        let (m, k) = estimate_parameters(trigrams.len() as u64, 0.6);
        match self.shards.par_iter_mut().find_any(|s| s.get_m() == m && s.get_k() == k) {
            None => {
                let mut shard = Shard::new(m, k);
                shard.add_message(message, &trigrams, &self.conn);
                self.shards.push(shard);
            }
            Some(shard) => { shard.add_message(message, &trigrams, &self.conn) }
        };
    }

    pub fn search(&self, query: &str) -> Vec<String> {
        if query.len() < 3 { return vec![]; }
        let query_words: Vec<&str> = query.split(" ").collect();
        let mut results: Vec<_> = self.shards
            .par_iter()
            .flat_map(|shard| shard.search(query, &self.conn))
            .filter_map(|result| {
                if query_words.par_iter().all(|q| result.contains(q)) {
                    Some(result)
                } else {
                    None
                }
            })
            .collect();
        results.truncate(5000);
        results
    }

    pub fn get_size(&self) -> usize {
        let x: Vec<_> = self.shards.iter().map(|s| s.size.clone()).collect();
        x.iter().sum()
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


