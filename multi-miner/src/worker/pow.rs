use super::{Worker, WorkerMessage};
use crossbeam_channel::{Receiver, Sender};
use rand::{
    distributions::{self as dist, Distribution as _},
    thread_rng,
};
use serde_derive::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use crate::job_template::{ProofMulti, JobTemplate, Hash,Task};
use log::{info, error, warn, debug};
use super::Seal;


pub struct Dummy {
    start: bool,
    task: Option<Task>,
    seal_tx: Sender<(String, Seal)>,
    worker_rx: Receiver<WorkerMessage>,
}

impl Dummy {
    pub fn new(
        seal_tx: Sender<(String, Seal)>,
        worker_rx: Receiver<WorkerMessage>,
    ) -> Self {
        Self {
            start: true,
            task: None,
            seal_tx,
            worker_rx,
        }
    }

    fn poll_worker_message(&mut self) {
        if let Ok(msg) = self.worker_rx.recv() {
            match msg {
                WorkerMessage::NewWork(task) => self.task = Some(task),
                WorkerMessage::Stop => {
                    self.start = false;
                }
                WorkerMessage::Start => {
                    self.start = true;
                }
            }
        }
    }

    fn solve(&self, task: Task, nonce: u64) {
        let seal = Seal {post_hash:Hash::random(), nonce };
        println!("solve send_seal_tx: {:?}", task);

        if let Err(err) = self.seal_tx.send((task.work_id.clone(), seal)) {
            error!("seal_tx send error {:?}", err);
        }
    }
}

impl Worker for Dummy {
    fn run<G: FnMut() -> u64>(&mut self, mut rng: G) {
        println!("thsi is worker thread id {:?}",thread::current().id());

        loop {
            self.poll_worker_message();
            if self.start {
                if let Some(task) = self.task.clone() {
                    self.solve(task, rng());
                }
            }
        }
    }
}