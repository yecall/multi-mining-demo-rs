pub mod client;
pub mod config;
pub mod job_template;
use crate::job_template::{ProofMulti,JobTemplate,Hash};
use crate::config::WorkerConfig;
pub mod worker;
pub mod miner;


pub struct Work {
    rawHash:Hash,
}
