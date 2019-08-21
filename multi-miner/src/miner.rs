use crate::client::{Client,Rpc};
use crate::config::WorkerConfig;
use crate::worker::{start_worker, WorkerController, WorkerMessage};
use crate::Work;
use crossbeam_channel::{select, unbounded, Receiver};
use std::sync::Arc;
use std::thread;
use log::{info,error,warn,debug};
use crate::worker::Seal;
use crate::job_template::{ProofMulti,JobTemplate,Hash,Task};
use lru_cache::LruCache;
use util::Mutex;
use crate::WorkMap;

const WORK_CACHE_SIZE: usize = 32;

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
            println!("now  check_seal: {}", work_id);

            let job = ProofMulti {
                extra_data: vec![],
                merkle_root: Hash::random(),
                nonce: 0,
                shard_num: 0,
                shard_cnt: 0,
                merkle_proof: vec![]
            };
            self.client.submit_job(Hash::random(), &job,Rpc::new("127.0.0.1:3131".parse().expect("valid rpc url")));
            //self.client.try_update_job_template();
            //self.notify_workers(WorkerMessage::Start);
        }

    }

    fn notify_workers(&self, message: WorkerMessage) {
            self.worker_controller.send_message(message.clone());

    }
}
