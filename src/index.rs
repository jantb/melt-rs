use sqlite::Connection;
use crate::bloom::estimate_parameters;
use crate::message::Message;
use crate::shard::Shard;

pub struct SearchIndex {
    shards: Vec<Shard>,
    conn: Connection,
}

impl SearchIndex {
    pub fn new() -> Self {
        let buf = dirs::home_dir().unwrap().into_os_string().into_string().unwrap();
        let index = Self {
            shards: vec![],
            conn: sqlite::open(format!("{}.data.sqlite", buf)).unwrap(),
        };
        let query = "CREATE TABLE if not exists data
(
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    value TEXT NOT NULL
);";
        index.conn.execute(query).unwrap();
        index
    }
    pub fn new_inmem() -> Self {
        let index = Self {
            shards: vec![],
            conn: sqlite::open(":memory:").unwrap(),
        };
        let query = "CREATE TABLE if not exists data
(
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    value TEXT NOT NULL
);";
        index.conn.execute(query).unwrap();
        index
    }

    pub fn add_message(&mut self, message: &Message) {
        let trigrams = message.get_trigram();
        let (m, k) = estimate_parameters(trigrams.len() as u64, 0.001);

        match self.shards.iter_mut().find(|s| s.get_m() == m && s.get_k() == k) {
            None => {
                let mut shard = Shard::new(m, k);
                shard.add_message(message, trigrams, &self.conn);
                self.shards.push(shard);
            }
            Some(shard) => { shard.add_message(message, trigrams, &self.conn) }
        };
    }

    pub fn search(&self, query: &str) -> Vec<Message> {
        let messages: Vec<Message> = self.shards.iter().map(|s| s.search(query, &self.conn)).flatten().filter(|s| {
            query.split(" ").all(|q| s.value.contains(q))
        }).collect();
        return messages;
    }
}

#[test]
fn test_search() {
    let mut index = SearchIndex::new_inmem();
    index.add_message(&Message { json: false, value: "notth".to_string() });
    index.add_message(&Message { json: false, value: "hello".to_string() });

    let result = index.search("llo");
    assert_eq!("hello", result.last().unwrap().value);

    let result = index.search("notth");
    assert_eq!("notth", result.last().unwrap().value);
}


