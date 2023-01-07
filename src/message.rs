use crate::trigrams::trigram;

#[derive(Copy, Clone)]
pub struct Message {
    pub json: bool,
    pub value: &'static str,
}


impl Message {
    pub fn get_trigram(&self) -> Vec<String> {
        return trigram(&self.value)
    }

    pub fn get_value(&self) -> &'static str {
        return self.value
    }
}