use crate::Work;
use super::config::ClientConfig;
use crate::job_template::{ProofMulti,JobTemplate,Hash};
use yee_jsonrpc_types::{
    error::Error as RpcFail, error::ErrorCode as RpcFailCode, id::Id, params::Params,
    request::MethodCall, response::Output, version::Version, };
use yee_stop_handler::{SignalSender, StopHandler};
use crossbeam_channel::Sender;
use failure::Error;
use futures::sync::{mpsc, oneshot};
use hyper::error::Error as HyperError;
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::rt::{self, Future, Stream};
use hyper::Uri;
use hyper::{Body, Chunk, Client as HttpClient, Method, Request};
use serde_json::error::Error as JsonError;
use serde_json::{self, json, Value};
use std::convert::Into;
use std::thread;
use std::time;
use log::{info,error,warn,debug};

type RpcRequest = (oneshot::Sender<Result<Chunk, RpcError>>, MethodCall);

#[derive(Debug)]
pub enum RpcError {
    Http(HyperError),
    Canceled, //oneshot canceled
    Json(JsonError),
    Fail(RpcFail),
}

#[derive(Debug, Clone)]
pub struct Rpc {
    sender: mpsc::Sender<RpcRequest>,
    stop: StopHandler<()>,
}

impl Rpc {
    pub fn new(url: Uri) -> Rpc {
        let (sender, receiver) = mpsc::channel(65_535);
        let (stop, stop_rx) = oneshot::channel::<()>();

        let thread = thread::spawn(move || {
            let client = HttpClient::builder().keep_alive(true).build_http();

            let stream = receiver.for_each(move |(sender, call): RpcRequest| {
                let req_url = url.clone();
                let request_json = serde_json::to_vec(&call).expect("valid rpc call");

                let mut req = Request::new(Body::from(request_json));
                *req.method_mut() = Method::POST;
                *req.uri_mut() = req_url;
                req.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

                let request = client
                    .request(req)
                    .and_then(|res| res.into_body().concat2())
                    .then(|res| sender.send(res.map_err(RpcError::Http)))
                    .map_err(|_| ());

                rt::spawn(request);
                Ok(())
            });

            rt::run(stream.select2(stop_rx).map(|_| ()).map_err(|_| ()));
        });

        Rpc {
            sender,
            stop: StopHandler::new(SignalSender::Future(stop), thread),
        }
    }

    pub fn request(
        &self,
        method: String,
        params: Vec<Value>,
    ) -> impl Future<Item = Output, Error = RpcError> {
        let (tx, rev) = oneshot::channel();

        let call = MethodCall {
            method,
            params: Params::Array(params),
            jsonrpc: Some(Version::V2),
            id: Id::Num(0),
        };

        let req = (tx, call);
        let mut sender = self.sender.clone();
        let _ = sender.try_send(req);
        rev.map_err(|_| RpcError::Canceled)
            .flatten()
            .and_then(|chunk| serde_json::from_slice(&chunk).map_err(RpcError::Json))
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        self.stop.try_send();
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub current_work_id: Option<Hash>,
    pub new_work_tx: Sender<Work>,
    pub config: ClientConfig,
    pub rpc: Rpc,
}

impl Client {
    pub fn new(new_work_tx: Sender<Work>, config: ClientConfig) -> Client {
        let uri: Uri = config.rpc_url.parse().expect("valid rpc url");

        Client {
            current_work_id: None,
            rpc: Rpc::new(uri),
            new_work_tx,
            config,
        }
    }

    fn send_submit_job_request(
        &self,
        rawHash: Hash,
        job: &ProofMulti,
    ) -> impl Future<Item = Output, Error = RpcError> {
        //let block: JsonBlock = block.into();
        let method = "submit_job".to_owned();
        let params = vec![json!(rawHash), json!(job)];

        self.rpc.request(method, params)
    }
    pub fn submit_job(&self, rawHash: Hash, job: &ProofMulti) {
        let future = self.send_submit_job_request(rawHash, job);
        if self.config.job_on_submit {
            let ret: Result<Option<Hash>, RpcError> = future.and_then(parse_response).wait();
            match ret {
                Ok(hash) => {
                    if hash.is_none() {
                        warn!(
                            "submit_job failed {}",
                            serde_json::to_string(job).unwrap()
                        );
                    }
                }
                Err(e) => {
                    error!("rpc call submit_job error: {:?}", e);
                }
            }
        }
    }

    pub fn poll_job_template(&mut self) {
        loop {
            info!("poll job template...");
            self.try_update_job_template();
            thread::sleep(time::Duration::from_millis(self.config.poll_interval));
        }
    }

    pub fn try_update_job_template(&mut self) {
        match self.get_job_template().wait() {
            Ok(job_template) => {
                if self.current_work_id != Some(job_template.rawHash) {
                    self.current_work_id = Some(job_template.rawHash.clone());
                    if let Err(e) = self.notify_new_work(job_template.clone()) {
                        error!("notify_new_job_template error: {:?}", e);
                    }
                }
            }
            Err(ref err) => {
                let is_method_not_found = if let RpcError::Fail(RpcFail { code, .. }) = err {
                    *code == RpcFailCode::MethodNotFound
                } else {
                    false
                };
                if is_method_not_found {
                    error!(
                        "RPC Method Not Found: \
                         please do checks as follow: \
                         1. if the  server has enabled the Miner API module; \
                         2. If the RPC URL for yee miner is right.",
                    );
                } else {
                    error!("rpc call get_job_template error: {:?}", err);
                }
            }
        }
    }

    fn get_job_template(&self) -> impl Future<Item = JobTemplate, Error = RpcError> {
        let method = "get_job_template".to_owned();
        let params = vec![];

        self.rpc.request(method, params).and_then(parse_response)
    }

    fn notify_new_work(&self, job_template: JobTemplate) -> Result<(), Error> {
       // let work: Work = job_template.into();
        let work = Work{ rawHash: job_template.rawHash};

        self.new_work_tx.send(work)?;
        Ok(())
    }
}

fn parse_response<T: serde::de::DeserializeOwned>(output: Output) -> Result<T, RpcError> {
    match output {
        Output::Success(success) => {
            serde_json::from_value::<T>(success.result).map_err(RpcError::Json)
        }
        Output::Failure(failure) => Err(RpcError::Fail(failure.error)),
    }
}
