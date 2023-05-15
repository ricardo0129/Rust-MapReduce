use std::{thread, time::Duration};
pub mod client;
pub mod helper;
use crate::{client::run, helper::map_reduce};
use crate::helper::{KeyValue,not_alphabetic};
use std::fs;

struct my_impl {}
impl map_reduce for my_impl {
    fn map(&self, _key: &String, contents: &String) -> Vec<KeyValue> {
        let mut kva: Vec<KeyValue> = vec![];
        for word in contents.split(not_alphabetic).filter(|&x| !x.is_empty()).map(String::from) {
            kva.push(KeyValue {key: word, value: "1".to_string() });
        }
        return kva;
    }
    fn reduce(&self, _key: &String, values: Vec<&String>) -> String {
        return values.iter().fold(0, |acc, x| acc + x.parse::<i32>().unwrap()).to_string();
    }
}
#[tokio::main]
async fn main() {
    let f = my_impl {};
    run(&f).await;
}
