


use yee_multi_miner::job_template::{ProofMulti,JobTemplate,Hash};

use std::thread;
use crossbeam_channel::unbounded;
use yee_multi_miner::config::MinerConfig;
use yee_multi_miner::client::Client;
use yee_multi_miner::config::ClientConfig;
use log::{info,error,warn,debug};
use yee_multi_miner::config::WorkerConfig;
use yee_multi_miner::miner::Miner;



pub struct Work {
    rawHash:Hash,

}
fn main() {

    let (new_work_tx, new_work_rx) = unbounded();
    let cc = ClientConfig {
        rpc_url: "http://127.0.0.1:3131/".to_owned(),
        poll_interval: 3000,
        job_on_submit: true
    };

    let workerc = WorkerConfig{ threads: 1 };

    let mut client = Client::new(new_work_tx, cc.clone());
    let mut miner =  Miner::new(client.clone(),new_work_rx,workerc.clone());
    info!("{}","start client ");

    let t= thread::Builder::new()
        .name("client".to_string())
        .spawn(move || client.poll_job_template())
        .expect("Start client failed!");

    miner.run();

}