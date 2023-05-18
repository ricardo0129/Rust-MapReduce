pub mod helper;
pub mod map_reduce;
use crate::map_reduce::MyImpl;
use crate::helper::MapReduce as other_map_reduce;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::BufWriter;

fn main() {
    //A sequential Map Reduce for Testing the distributed Version
    //Currently The Map Reduce implementation is reciding in helper.rs
    let obj: &dyn other_map_reduce = &MyImpl {};
    let mut kva: Vec<helper::KeyValue> = vec![];
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    for file in args.iter() {
        let contents = fs::read_to_string(file)
            .expect("error opening file");
        kva.append(&mut obj.map(file, &contents));
    }
    let f = "mr-out".to_string();
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(f).unwrap();
    let mut writer = BufWriter::new(file);
    kva.sort_by(|s1, s2| s1.key.cmp(&s2.key));
    let mut i: usize = 0;
    while i<kva.len() {
        let mut values: Vec<&String> = vec![];
        let mut j: usize = i;
        while j<kva.len() && kva[i].key == kva[j].key {
            values.push(&kva[j].value);
            j += 1;
        }
        let value = obj.reduce(&kva[j-1].key, values);
        let data = format!("{} {}", &kva[j-1].key, value);
        writeln!(writer, "{}", data).unwrap();
        i = j;
    }
}
