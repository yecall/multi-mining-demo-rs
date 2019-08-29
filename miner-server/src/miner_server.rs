use jsonrpc_derive::rpc;
use jsonrpc_core::*;
use jsonrpc_http_server::*;
use jsonrpc_core::futures::future::{self, FutureResult};
use yee_multi_miner::job_template::{ProofMulti, JobTemplate, Hash, DifficultyType};
use std::{thread, time};
use futures::future::Future;
use log::{info,error,warn,debug};

#[rpc]
pub trait Rpc {
   // curl -H "Content-Type: application/json" -X POST --data '{"id": 2, "jsonrpc": "2.0", "method":"get_job_template"}' http://127.0.0.1:3131
    #[rpc(name = "get_job_template")]
    fn get_job_template(&self) -> Result<JobTemplate>;


    #[rpc(name = "get_job_template_async")]
    fn call(&self) -> FutureResult<JobTemplate, Error>;


    //curl -H "Content-Type: application/json" -X POST --data '{"id": 2, "jsonrpc": "2.0", "method":"submit_job","params":["0x09d000e581f9fb4de8a2e37c09c85c5ed8ae825eaa5845363c8b780bd6b84e1e",{"extra_data":[3,3,3],"merkle_root":"0x09d000e581f9fb4de8a2e37c09c85c5ed8ae825eaa5845363c8b780bd6b84e1e","nonce":33,"shard_num":2, "shard_cnt":4,"merkle_proof":[3,3,3]}]}' http://127.0.0.1:3131
     #[rpc(name = "submit_job")]
    fn submit_job(&self, rawHash: Hash,job:ProofMulti) -> Result<Hash> ;

}
struct RpcImpl;

impl Rpc for RpcImpl {
    fn get_job_template(&self) -> Result<JobTemplate> {
        let job = JobTemplate { difficulty: DifficultyType::from(0x00003fff) << 224, rawHash: Hash::random() };
        //0x00003fff   0x3fffffff
        Ok(job)
    }


    fn call(&self, ) -> FutureResult<JobTemplate, Error> {
        let ten_millis = time::Duration::from_millis(2000);
        thread::sleep(ten_millis);
        let job = JobTemplate { difficulty: DifficultyType::from(0x00fffffff) << 224, rawHash: Hash::random() };

        future::ok(job).into()
    }

    fn submit_job(&self, rawHash: Hash,job:ProofMulti) -> Result<Hash> {

        info!("{}","new submit");

        Ok(rawHash)
    }
}

pub fn http_run(url:String) {

    let mut io = IoHandler::new();
    let rpc = RpcImpl;

    io.extend_with(rpc.to_delegate());

    let _server = ServerBuilder::new(io)
       // .start_http(&"127.0.0.1:3131".parse().unwrap())
        .start_http(&url.parse().unwrap())
        .expect("Unable to start RPC server");

    _server.wait();

}

