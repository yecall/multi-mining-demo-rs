mod miner_server;
use crate::miner_server::http_run;
use std::thread;

fn main() {

    http_run();

    println!("{}", "miner！！！");

}