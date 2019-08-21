use yee_multi_miner::job_template::{ProofMulti,JobTemplate,Hash};
use std::thread;
use crossbeam_channel::unbounded;
use yee_multi_miner::client::Client;
use log::{info,error,warn,debug};
use yee_multi_miner::miner::Miner;
use yee_multi_miner::config::{WorkerConfig,NodeConfig,MinerConfig,ClientConfig};
use std::collections::HashMap;
use yee_multi_miner::gateway::Gateway;


fn main() {
    println!("thsi is main thread id {:?}",thread::current().id());

    let mut  map =  HashMap::new();
    map.insert("0".to_string(), "http://127.0.0.1:3131".to_string());
    map.insert("1".to_string(), "http://127.0.0.1:4131".to_string());
    map.insert("2".to_string(), "http://127.0.0.1:5131".to_string());
    map.insert("3".to_string(), "http://127.0.0.1:6131".to_string());



    let (new_work_tx, new_work_rx) = unbounded();

    let cc = ClientConfig {
        poll_interval: 1000,
        job_on_submit: true
    };
    
    let workert = WorkerConfig{ threads: 1 };

    let  client = Client::new( cc.clone());

    let mut gateway = Gateway::new(client.clone(),new_work_tx,map);

    let mut miner =  Miner::new(client.clone(),new_work_rx,workert.clone());
    info!("{}","start client ");

    let t= thread::Builder::new()
        .name("client".to_string())
        .spawn(move || gateway.poll_job_template())
        .expect("Start client failed!");

    miner.run();

}