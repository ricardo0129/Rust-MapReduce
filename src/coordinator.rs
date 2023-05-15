struct Task {
    id: i32, 
    status: i32
}

struct Coordinator {
   mapTasks: Vec<Task>,
   mapTasksLeft: i32,
   reduceTasks: Vec<Task>,
   reduceTasksLeft: i32
}

impl Coordinator {
}

fn main() {
    println!("TEST")
}
