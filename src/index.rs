use std::fs::File;
use std::io::{BufReader, Error};
use rusqlite::{Connection, OpenFlags};
use crate::bloom::estimate_parameters;
use crate::message::Message;
use crate::shard::Shard;

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
        let serialized = serde_json::to_vec(self.shards.iter().as_slice())?;
        let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
        let path = format!("{}/.melt_index{}.sqlite", buf,self.thread);

        std::fs::write(path, serialized)?;
        Ok(())
    }

    pub fn load_from_json(thread: u8) -> Self {

        let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
        let path = format!("{}/.melt_index{}.sqlite", buf,thread);
        let file = File::open(path);

        match file {
            Ok(file) => {
                let reader = BufReader::new(file);
                let shards: Vec<Shard> = serde_json::from_reader(reader).unwrap_or(vec![]);
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

    pub fn add_message(&mut self, message: &Message) {
        let trigrams = message.get_trigram();
        let (m, k) = estimate_parameters(trigrams.len() as u64, 0.1);

        match self.shards.iter_mut().find(|s| s.get_m() == m && s.get_k() == k) {
            None => {
                let mut shard = Shard::new(m, k);
                shard.add_message(message, &trigrams, &self.conn);
                self.shards.push(shard);
            }
            Some(shard) => { shard.add_message(message, &trigrams, &self.conn) }
        };
    }

    pub fn search(&self, query: &str) -> Vec<Message> {
        if query.len() < 3 { return vec![]; };
        return self.shards.iter().map(|s| s.search(query, &self.conn)).flatten().filter(|s| {
            query.split(" ").all(|q| s.value.contains(q))
        }).collect();
    }

    pub fn get_size(&self) -> usize {
        let x: Vec<_> = self.shards.iter().map(|s| s.size.clone()).collect();
        x.iter().sum()
    }
}

#[test]
fn test_search() {
    let mut index = SearchIndex::new_in_mem();
    index.add_message(&Message { json: false, value: "notth".to_string() });
    index.add_message(&Message { json: false, value: "hello".to_string() });

    let result = index.search("llo");
    assert_eq!("hello", result.last().unwrap().value);

    let result = index.search("notth");
    assert_eq!("notth", result.last().unwrap().value);
}


