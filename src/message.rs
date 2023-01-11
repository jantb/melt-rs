use crate::trigrams::trigram;

#[derive(Clone)]
pub struct Message {
    pub json: bool,
    pub value: String,
}

impl Message {
    pub fn get_trigram(&self) -> Vec<String> {
        return trigram(&self.value);
    }

    pub fn get_value(&self) -> String {
        return self.value.clone();
    }
}