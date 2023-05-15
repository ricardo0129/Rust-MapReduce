use serde::{Serialize, Serializer, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String
}

pub fn not_alphabetic(c: char) -> bool {
    return !c.is_alphabetic();
}

/*
pub fn map(_key: &String, contents: &String) -> Vec<KeyValue> {
    //key: document name
    //value: document contents
    let mut kva: Vec<KeyValue> = vec![];
    for word in contents.split(not_alphabetic).filter(|&x| !x.is_empty()).map(String::from) {
        kva.push(KeyValue {key: word, value: "1".to_string() });
    }
    return kva;
}

pub fn reduce(_key: &String, values: Vec<&String>) -> String {
    //key: a word
    //values: a list of counts 
    return values.iter().fold(0, |acc, x| acc + x.parse::<i32>().unwrap()).to_string();
}
*/

pub trait map_reduce {
    fn map(&self, key: &String, contents: &String) -> Vec<KeyValue>;
    fn reduce(&self, key: &String, values: Vec<&String>) -> String;
    //fn add(&self, key:i32) -> i32;
}

