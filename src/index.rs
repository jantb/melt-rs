use crate::bloom::estimate_parameters;
use crate::message::Message;
use crate::shard::Shard;

pub struct SearchIndex {
    shards: Vec<Shard>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            shards: vec![],
        }
    }

    pub fn add_message(&mut self, message: &Message) {
        let trigrams = message.get_trigram();
        let (m, k) = estimate_parameters(trigrams.len() as u64, 0.001);

        match self.shards.iter_mut().find(|s| s.get_m() == m && s.get_k() == k) {
            None => {
                let mut shard = Shard::new(m, k);
                shard.add_message(message, trigrams);
                self.shards.push(shard);
            }
            Some(shard) => { shard.add_message(message, trigrams) }
        };
    }

    pub fn search(&self, query: &str) -> Vec<Message> {
        let messages: Vec<Message> = self.shards.iter().map(|s| s.search(query)).flatten().filter(|s| {
            query.split(" ").all(|q| s.value.contains(q))
        }).collect();
        return messages;
    }
}

#[test]
fn test_search() {
    let mut index = SearchIndex { shards: vec![] };
    index.add_message(&Message { json: false, value: "notth".to_string() });
    index.add_message(&Message { json: false, value: "hello".to_string() });

    let result = index.search("llo");
    assert_eq!("hello", result.last().unwrap().value);

    let result = index.search("notth");
    assert_eq!("notth", result.last().unwrap().value);
}


