use rpc::coordinator_client::CoordinatorClient;
use rpc::{RequestTaskArg, RequestTaskReply, CompleteTaskArg, CompleteTaskReply};
use std::{thread, time::Duration};
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use crate::helper::{MapReduce, KeyValue};


pub fn ihash(key: &String) -> i32 {
    //maybe something better ?
    let mut hasher = DefaultHasher::new();
    hasher.write(key.as_bytes());
    (hasher.finish() & 0x7fffffff) as i32
}

#[derive(Serialize, Deserialize)]
pub struct Obj {
    all: Vec<KeyValue>
}

fn split_string(line: &String) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

pub mod rpc{
    tonic::include_proto!("rpc");
}

async fn request_task() -> RequestTaskReply {
    let mut client = CoordinatorClient::connect("http://[::1]:50051").await.unwrap();

    let request = tonic::Request::new(RequestTaskArg {
        name: "haha".into(),
    });

    let response = client.say_hello(request).await.unwrap();
    return response.into_inner();
}

async fn completed_task(task_type: i32, task_id: i32, file_name: String) -> CompleteTaskReply {
    let mut client = CoordinatorClient::connect("http://[::1]:50051").await.unwrap();

    let request = tonic::Request::new(CompleteTaskArg {
        task_type: task_type.to_string(),
        task_id: task_id.to_string(),
        file_names: file_name
    });

    let response = client.completed_task(request).await.unwrap();
    return response.into_inner();
}

async fn process_map(obj: &dyn MapReduce, file_names: &String, n_reduce: i32, t_id: i32) {
    //Typically Only one but to make it symmetric to Reduce
    println!("Process Map {} {}", file_names, t_id);
    let files: Vec<String> = split_string(file_names);
    let mut kva: Vec<KeyValue> = vec![];
    for file in files.iter() {
        let contents = fs::read_to_string(file)
            .expect("error openning file");
        kva.append(&mut obj.map(file, &contents));
    }
    let mut open_files: Vec<BufWriter<File>> = vec![];
    let file_names: String = "".to_string();
    let mut json_obs: Vec<Obj> = vec![];
    for i in 0..n_reduce {
        let f = format!("mr-{}-{}", t_id, i);
        let file = File::create(f).unwrap();
        let writer = BufWriter::new(file);
        open_files.push(writer);
        json_obs.push(Obj {all: vec![]});
    }
    for pair in kva {
        json_obs[(ihash(&pair.key)%n_reduce) as usize].all.push(pair);
    }

    for i in 0..n_reduce {
        serde_json::to_writer(&mut open_files[i as usize], &json_obs[i as usize]).unwrap();
        writeln!(open_files[i as usize]).unwrap();
    }
    println!("{}", file_names);
    completed_task(1, t_id, file_names).await;  
}

async fn process_reduce(obj: &dyn MapReduce, _file_names: &String, t_id: i32, n_map: i32) {
    //TODO support a stream of file_names
    //Currently I use a special convention of filenames depedent on n_map and t_id
    
    let mut kva: Vec<KeyValue> = vec![];
    for i in 0..n_map {
        let f = format!("mr-{}-{}", i, t_id);
        let data = fs::read_to_string(f).expect("Unable to read file");
        //println!("{}", data);
        let mut res: Obj = serde_json::from_str(&data).unwrap();
        kva.append(&mut res.all);
    }
    let f = format!("mr-out-{}", t_id);
    let file = OpenOptions::new()
        .create_new(true)
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
        //println!("{} {}", i, data);
        //file.write_all(data.as_bytes()).expect("Unable to write data");
        writeln!(writer, "{}", data).unwrap();
        i = j;
    }
    completed_task(2, t_id, "".to_string()).await;  
}

pub async fn run(obj: &dyn MapReduce) {
    loop {
        let res = request_task().await;
        if res.task_type == "1".to_string() {
            println!("T1");
            process_map(obj, &res.file_names, res.n_reduce.parse::<i32>().unwrap(), res.task_id.parse::<i32>().unwrap()).await;
        }
        else if res.task_type == "2".to_string() {
            println!("T2");
            process_reduce(obj, &res.file_names, res.task_id.parse::<i32>().unwrap(), res.n_map.parse::<i32>().unwrap()).await;
        }
        else {
            println!("No Work");
        }
        thread::sleep(Duration::from_millis(1000));
    }
}
