use crate::client::Client;
use crate::config::WorkerConfig;
use crate::worker::{start_worker, WorkerController, WorkerMessage};
use crate::Work;
use crossbeam_channel::{select, unbounded, Receiver};
use std::sync::Arc;
use std::thread;
use log::{info,error,warn,debug};
use crate::worker::Seal;
use crate::job_template::{ProofMulti,JobTemplate,Hash};
use lru_cache::LruCache;
use util::Mutex;
const WORK_CACHE_SIZE: usize = 32;

pub struct Miner {
    pub client: Client,
    pub worker_controller: WorkerController,
    pub work_rx: Receiver<Work>,
    pub seal_rx: Receiver<(Hash, Seal)>,
    pub works: Mutex<LruCache<Hash, Work>>,


}

impl Miner {
    pub fn new(
        client: Client,
        work_rx: Receiver<Work>,
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
                        let pow_hash = work.rawHash;
                        println!("cache_and send_WorkerMessage: {}", pow_hash);
                         self.works.lock().insert(pow_hash.clone(), work);

                        self.notify_workers(WorkerMessage::NewWork(pow_hash));
                    },
                    _ => {
                        error!("work_rx closed");
                        break;
                    },
                },
                recv(self.seal_rx) -> msg => match msg {
                    Ok((pow_hash, seal)) => self.check_seal(pow_hash, seal),
                    _ => {
                        error!("seal_rx closed");
                        break;
                    },
                }
            };
        }
    }

    fn check_seal(&mut self, pow_hash: Hash, seal: Seal) {
        if let Some(work) = self.works.lock().get_refresh(&pow_hash) {
            println!("now  check_seal: {}", pow_hash);

            let job = ProofMulti {
                extra_data: vec![],
                merkle_root: pow_hash.clone(),
                nonce: 0,
                shard_num: 0,
                shard_cnt: 0,
                merkle_proof: vec![]
            };
            self.client.submit_job(pow_hash, &job);
            //self.client.try_update_job_template();
            //self.notify_workers(WorkerMessage::Start);
        }

    }

    fn notify_workers(&self, message: WorkerMessage) {
            self.worker_controller.send_message(message.clone());

    }
}
