mod miner_server;
use crate::miner_server::http_run;
use std::thread;

fn main() {

    let thread_one = thread::spawn(|| http_run("127.0.0.1:3131".to_string()));

    let thread_two = thread::spawn(|| http_run("127.0.0.1:4131".to_string()));

    let thread_three = thread::spawn(|| http_run("127.0.0.1:5131".to_string()));

    let thread_four = thread::spawn(|| http_run("127.0.0.1:6131".to_string()));

    thread_one.join();
    thread_two.join();
    thread_three.join();
    thread_four.join();
    

}