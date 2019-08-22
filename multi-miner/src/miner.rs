use crate::client::{Client,Rpc};
use crate::config::WorkerConfig;
use crate::worker::{start_worker, WorkerController, WorkerMessage};
use crate::Work;
use crossbeam_channel::{select, unbounded, Receiver};
use std::sync::Arc;
use std::thread;
use log::{info,error,warn,debug};
use crate::worker::Seal;
use crate::job_template::{ProofMulti,JobTemplate,Hash,Task,DifficultyType};
use lru_cache::LruCache;
use util::Mutex;
use crate::WorkMap;
use std::collections::HashMap;
use std::convert::TryInto;
use core::borrow::{BorrowMut, Borrow};

const WORK_CACHE_SIZE: usize = 32;
/// Max length in bytes for pow extra data
pub const MAX_EXTRA_DATA_LENGTH: usize = 32;

pub struct Miner {
    pub client: Client,
    pub worker_controller: WorkerController,
    pub work_rx: Receiver<WorkMap>,
    pub seal_rx: Receiver<(String, Seal)>,
    pub works: Mutex<LruCache<String, WorkMap>>,


}

impl Miner {
    pub fn new(
        client: Client,
        work_rx: Receiver<WorkMap>,
        worker: WorkerConfig,
    ) -> Miner {
        let (seal_tx, seal_rx) = unbounded();

        let worker_controller = start_worker(worker,seal_tx.clone());

        Miner {
            works: Mutex::new(LruCache::new(WORK_CACHE_SIZE)),
            client,
            worker_controller,
            work_rx,
            seal_rx,
        }
    }

    pub fn run(&mut self) {
        println!("thsi is miner run thread id {:?}",thread::current().id());

        loop {
            println!("thsi is miner run  loop");
            select! {
                recv(self.work_rx) -> msg => match msg {
                    Ok(work) => {
                        let work_id = work.work_id.clone();
                        println!("cache_and send_WorkerMessage: {}", work_id);
                        self.works.lock().insert(work_id.clone(), work);

                        let task = Task{
                                    work_id: work_id,
                                    extra_data: vec![],
                                    merkle_root: Hash::random()
                                   };

                        self.notify_workers(WorkerMessage::NewWork(task));
                    },
                    _ => {
                        error!("work_rx closed");
                        break;
                    },
                },
                recv(self.seal_rx) -> msg => match msg {
                    Ok((work_id, seal)) => self.check_seal(work_id, seal),
                    _ => {
                        error!("seal_rx closed");
                        break;
                    },
                }
            };
        }
    }

    fn check_seal(&mut self, work_id: String, seal: Seal) {
        if let Some(work) = self.works.lock().get_refresh(&work_id) {
            println!("now  check_seal  work_id: {}", work_id);

            let mut work_set = &work.work_map;
            let mut work_change:HashMap<String,Work> =  HashMap::new();

            let mut f =false;
            let mut i = 0;
            let len = work_set.len();

            for (key, value) in  work_set {
                let w = Work{
                    rawHash: value.rawHash.clone(),
                    difficulty: value.difficulty.clone(),
                    extra_data: value.extra_data.clone(),
                    merkle_root: value.merkle_root.clone(),
                    merkle_proof: value.merkle_proof.clone(),
                    shard_num: value.shard_num.clone(),
                    shard_cnt: value.shard_cnt.clone(),
                    has_commit: value.has_commit.clone()
                };


                //只要有一个work的has_commit为true，则缓存lru更新work
                if value.has_commit{
                    f=true;
                    i = i+1;
                };

                work_change.insert(key.to_string(),w);

                let t =  self.verify_target(seal.post_hash,value.difficulty,value.extra_data.clone());
                let m =  self.verify_merkel_proof(value.merkle_root,value.merkle_proof.clone());

                if(t&&m&&!value.has_commit){
                    let submitjob = ProofMulti {
                        extra_data: value.extra_data.clone(),
                        merkle_root: value.merkle_root.clone(),
                        nonce: seal.nonce,
                        shard_num: value.shard_num.clone(),
                        shard_cnt: value.shard_cnt.clone(),
                        merkle_proof: value.merkle_proof.clone()
                    };
                    println!("find seal ,now  submit_job  work_id: {:?}", submitjob);

                    self.client.submit_job(value.rawHash, &submitjob,Rpc::new("127.0.0.1:3131".parse().expect("valid rpc url")));
                }

            }

            if i >= len{//所有分片都挖出同时没有新job
                self.notify_workers(WorkerMessage::Stop);
            }
            while f {
                self.works.lock().insert(work_id.clone(), WorkMap{ work_id:work_id.clone(), work_map:work_change.clone() });
            }
        }

    }

    fn notify_workers(&self, message: WorkerMessage) {
            self.worker_controller.send_message(message.clone());

    }

    fn verify_target(&self,hash:Hash,difficulty:DifficultyType, extra_data: Vec<u8>)-> bool{

        let proof_difficulty = DifficultyType::from(hash.as_ref());

        if extra_data.len() > MAX_EXTRA_DATA_LENGTH || proof_difficulty > difficulty{
            return false;
        }
        return true;
    }

    fn verify_merkel_proof(&self,merkle_root:Hash,merkle_proof: Vec<u8>)-> bool{

        return true;
    }


}
