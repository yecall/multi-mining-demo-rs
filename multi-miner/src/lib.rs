pub mod client;
pub mod config;
pub mod job_template;
use crate::job_template::{ProofMulti,JobTemplate,Hash,DifficultyType};
use crate::config::WorkerConfig;
pub mod worker;
pub mod miner;
pub mod merkle;
pub mod gateway;
use std::collections::HashMap;
use yee_merkle::proof::Proof;

#[derive(Clone,  Debug)]
pub struct Work {
    pub rawHash:Hash,
    pub difficulty: DifficultyType,
    /// Extra Data used to encode miner info AND more entropy
    pub extra_data: Vec<u8>,
    /// merkle root of multi-mining headers
    pub merkle_root: Hash,
    /// merkle tree spv proof
    pub merkle_proof:Proof<[u8;32]>,
    /// shard info
    pub shard_num: u32,
    pub shard_cnt: u32,
    //if commit ,set has_commit =true;
   // pub has_commit: bool,
}
#[derive(Clone, Debug)]
pub struct WorkMap {
    pub work_id: String,
    pub merkle_root: Hash,
    pub extra_data: Vec<u8>,
    pub work_map: HashMap<String,Work>,

}
