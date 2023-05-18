use tonic::{transport::Server, Request, Response, Status};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use std::env;
use std::process;

use rpc::coordinator_server::{Coordinator, CoordinatorServer};
use rpc::{RequestTaskArg, RequestTaskReply, CompleteTaskArg, CompleteTaskReply};

pub mod rpc {
    tonic::include_proto!("rpc");
}

pub struct Task {
    file_name: String,
    //t_type: i32, //0 invalid, 1 map taks, 2 reduce task
    status: i32, //0 unassigned, 1 inprogress, 2 completed
}

pub struct MyCoordinator {
    flag: Arc<Mutex<i32>>,
    shared: Arc<Shared>,
}
struct Shared {
    map_total: Mutex<i32>,
    reduce_total: Mutex<i32>, 
    map_tasks: Mutex<Vec<Task>>,
    map_left: Mutex<i32>,
    reduce_tasks: Mutex<Vec<Task>>,
    reduce_left: Mutex<i32>,
}

impl MyCoordinator {
    pub fn done(&self) -> bool {
        //return self.map_left.lock().unwrap() + self.reduce_left.lock().unwrap() == 0;
        let map_left = self.shared.map_left.lock().unwrap();
        let reduce_left = self.shared.reduce_left.lock().unwrap();
        return *map_left + *reduce_left == 0;
    }
}

#[tonic::async_trait]
impl Coordinator for MyCoordinator {
    async fn say_hello(
        &self,
        request: Request<RequestTaskArg>,
    ) -> Result<Response<RequestTaskReply>, Status> {
        //println!("Got a request: {:?}", request);
        println!("{}", request.into_inner().name);

        //Try to find a map task to assign
        //TODO replace this with a more efficient search 
        //TODO Try to get rid of all Mutex and replace with 1 Mutex if possible
        let mut reply = RequestTaskReply::default();
            
        for i in 0..*self.shared.map_total.lock().unwrap() {
            if self.shared.map_tasks.lock().unwrap()[i as usize].status == 0 {
                reply.task_type = "1".to_string();
                reply.file_names = self.shared.map_tasks.lock().unwrap()[i as usize].file_name.clone();
                reply.n_reduce = self.shared.reduce_total.lock().unwrap().to_string();
                reply.task_id = i.to_string();
                return Ok(Response::new(reply));
            }
        }

        for i in 0..*self.shared.reduce_total.lock().unwrap() {
            if self.shared.reduce_tasks.lock().unwrap()[i as usize].status == 0 {
                reply.task_type = "2".to_string();
                reply.file_names = self.shared.reduce_tasks.lock().unwrap()[i as usize].file_name.clone();
                reply.task_id = i.to_string();
                reply.n_map = self.shared.map_total.lock().unwrap().to_string();
                return Ok(Response::new(reply));
            }
        }

        reply.task_type = "-1".to_string();

        return Ok(Response::new(reply));
    }
    async fn completed_task(&self, request: Request<CompleteTaskArg>) -> Result<Response<CompleteTaskReply>, Status> {
        let req: CompleteTaskArg = request.into_inner();
        let task_id: i32 = req.task_id.parse::<i32>().unwrap();
        if req.task_type == "1".to_string() {
            //Completed a Map Task
            if self.shared.map_tasks.lock().unwrap()[task_id as usize].status != 2 {
                //Make sure the Tasks has not been set as completed
                self.shared.map_tasks.lock().unwrap()[task_id as usize].status = 2;
                *self.shared.map_left.lock().unwrap() -= 1;
            }
        }
        else if req.task_type == "2".to_string() {
            //Completed A Reduce Task
            if self.shared.reduce_tasks.lock().unwrap()[task_id as usize].status !=2 {
                self.shared.reduce_tasks.lock().unwrap()[task_id as usize].status = 2;
                *self.shared.reduce_left.lock().unwrap() -= 1;
            }
        }
        let reply = CompleteTaskReply {a: "1".to_string()};
        *self.flag.lock().unwrap() = self.done() as i32;
        Ok(Response::new(reply))
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut all_files: Vec<String> = env::args().collect();
    if all_files.len() < 3 {
        println!("Usage server n_reduce files ...")

    }
    all_files.remove(0);
    let n_reduce: i32 = all_files[0].parse::<i32>().unwrap();
    all_files.remove(0);

    let foo = Arc::new(Shared {map_left: Mutex::new(0), reduce_left: Mutex::new(0), map_tasks: Mutex::new(vec![]), reduce_tasks: Mutex::new(vec![]),
    map_total: Mutex::new(0), reduce_total: Mutex::new(0)});
    let coordinator : MyCoordinator = MyCoordinator {shared: foo, flag: Arc::new(Mutex::new(0))};

    let (_shutdown_trigger, shutdown_signal1) = triggered::trigger();

    *coordinator.shared.map_left.lock().unwrap() = all_files.len() as i32;
    *coordinator.shared.reduce_left.lock().unwrap() = n_reduce;
    *coordinator.shared.map_total.lock().unwrap() = all_files.len() as i32;
    *coordinator.shared.reduce_total.lock().unwrap() = n_reduce;

    for i in 0..all_files.len() {
        let t: Task = Task {file_name: all_files[i].clone(), status: 0};
        println!("file:{}", t.file_name);
        coordinator.shared.map_tasks.lock().unwrap().push(t);
    }

    for _i in 0..n_reduce {
        let t: Task = Task {file_name: "".to_string(), status: 0};
        coordinator.shared.reduce_tasks.lock().unwrap().push(t);
    }

    //let flag = Arc::new(Mutex::new(12));
    let flag = coordinator.flag.clone();

    let addr = "[::1]:50051".parse().unwrap();
    tokio::spawn(async move {
        Server::builder()
            .add_service(CoordinatorServer::new(coordinator))
            //.serve(addr)
            .serve_with_shutdown(addr, shutdown_signal1)
            .await.unwrap();
    });
    
    loop {
        //I hope No other Human Eyes lay open this 
        for _i in 0..1 {
            thread::sleep(Duration::from_millis(1000));
            let val = flag.lock().unwrap();
            println!("T11 {}", val);
            if *val == 1 {
                //shutdown_trigger.trigger();
                println!("All Tasks Completed");
                process::exit(1);
            }
        }
    }
}


#[tokio::main]
async fn main() {
    run().await.unwrap();
}
