use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String
}

pub fn not_alphabetic(c: char) -> bool {
    return !c.is_alphabetic();
}

pub trait MapReduce {
    fn map(&self, key: &String, contents: &String) -> Vec<KeyValue>;
    fn reduce(&self, key: &String, values: Vec<&String>) -> String;
    //fn add(&self, key:i32) -> i32;
}

