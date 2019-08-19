pub mod client;
pub mod config;
pub mod job_template;
use crate::job_template::{ProofMulti,JobTemplate,Hash,DifficultyType};
use crate::config::WorkerConfig;
pub mod worker;
pub mod miner;


pub struct Work {
    pub rawHash:Hash,
    pub difficulty: DifficultyType,
    /// Extra Data used to encode miner info AND more entropy
    pub extra_data: Vec<u8>,
    /// merkle root of multi-mining headers
    pub merkle_root: Hash,
    /// merkle tree spv proof
    pub merkle_proof: Vec<u8>,
    /// shard info
    pub shard_num: u32,
    pub shard_cnt: u32,
}
