use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinerConfig {
    pub client: ClientConfig,
    pub workers: Vec<WorkerConfig>,
    pub shards:Vec<ClientConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub rpc_url: String,
    pub poll_interval: u64,
    pub job_on_submit: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "worker_type")]
pub struct WorkerConfig {
    pub threads: usize,
}


//#[derive(Serialize, Deserialize)]
//#[derive(Debug, Clone)]
//pub struct Shards {
//    pub shards: HashMap<String, String>,
//}