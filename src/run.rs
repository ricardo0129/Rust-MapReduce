pub mod map_reduce;
pub mod client;
pub mod helper;
use crate::map_reduce::MyImpl;
use crate::client::run;

#[tokio::main]
async fn main() {
    let f = MyImpl {};
    run(&f).await;
}
