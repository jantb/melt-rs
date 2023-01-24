use std::fs;
use std::fs::File;
use std::io::{ Error, Read};
use bincode::deserialize;
use rusqlite::{Connection, OpenFlags};
use crate::bloom::estimate_parameters;
use crate::shard::Shard;
use crate::trigrams::trigram;

pub struct SearchIndex {
    shards: Vec<Shard>,
    thread: usize,
    conn: Connection,
}

fn default_conn(thread: u8) -> Connection {
    let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
    let path = format!("{}/.melt{thread}.sqlite", buf);
    let connection = Connection::open_with_flags(path,
                                                 OpenFlags::SQLITE_OPEN_READ_WRITE
                                                     | OpenFlags::SQLITE_OPEN_CREATE
                                                     | OpenFlags::SQLITE_OPEN_NO_MUTEX
                                                     | OpenFlags::SQLITE_OPEN_URI).unwrap();
    connection.execute("PRAGMA synchronous = OFF;", ()).unwrap();
    connection
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
                Self::create_table(index)
            }
        }
    }

    pub fn new_in_mem() -> Self {
        let index = Self {
            shards: vec![],
            conn: Connection::open_in_memory().unwrap(),
            thread: 0,
        };
        Self::create_table(index)
    }

    fn create_table(index: SearchIndex) -> SearchIndex {
        let query = "CREATE TABLE if not exists data
                        (
                            id   INTEGER PRIMARY KEY AUTOINCREMENT,
                            value TEXT NOT NULL
                        );";
        index.conn.execute(query, ()).unwrap();
        index
    }

    pub fn add_message(&mut self, message: &str) {
        let trigrams = trigram(message);
        let (m, k) = estimate_parameters(trigrams.len() as u64, 0.01);

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
}

fn get_file_as_byte_vec(filename: &String) -> Result<Vec<u8>, Error> {
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;

    Ok(buffer)
}

#[test]
fn test_search() {
    let mut index = SearchIndex::new_in_mem();
    index.add_message(&"notth");
    index.add_message(&"hello");

    let result = index.search("llo");
    assert_eq!("hello", result.last().unwrap());

    let result = index.search("notth");
    assert_eq!("notth", result.last().unwrap());
}


