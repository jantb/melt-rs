use std::fs;
use std::fs::File;
use std::io::{ Error, Read};
use std::sync::atomic::AtomicUsize;
use bincode::deserialize;
use rocksdb::{DB, DBWithThreadMode, SingleThreaded};
use crate::bloom::estimate_parameters;
use crate::shard::Shard;
use crate::trigrams::trigram;
pub static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct SearchIndex {
    shards: Vec<Shard>,
    thread: usize,
    conn: DBWithThreadMode<SingleThreaded>,
}

fn default_conn(thread: u8) -> DBWithThreadMode<SingleThreaded> {
    let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
    let path = format!("{}/.melt{thread}.db", buf);
    let db = DB::open_default(path).unwrap();
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
        let (m, k) = estimate_parameters(trigrams.len() as u64, 0.7);

        match self.shards.iter_mut().find(|s| s.get_m() == m && s.get_k() == k) {
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
        let mut results = vec![];

        for shard in self.shards.iter() {
            let shard_results = shard.search(query, &self.conn);
            for result in shard_results {
                if query_words.iter().all(|q| result.contains(q)) {
                    results.push(result);
                    if results.len() == 100 {
                        return results;
                    }
                }
            }
        }
        return results;
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


