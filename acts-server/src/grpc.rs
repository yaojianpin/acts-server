use acts::{Engine, Message, Workflow};
use acts_grpc::{
    acts_service_server::*, model::ActValue, ActionOptions, ActionState, MessageOptions, Vars,
    WorkflowMessage,
};
use futures::Stream;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{net::SocketAddr, pin::Pin, sync::Arc};
use time::macros::format_description;
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Code, Request, Response, Status};
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::EnvFilter;

type MessageStream = Pin<Box<dyn Stream<Item = Result<WorkflowMessage, Status>> + Send>>;
const ACTS_ENV_LOG: &str = "ACTS_LOG";
macro_rules! wrap_state_result {
    ($input: expr) => {
        match $input {
            Ok(state) => {
                let vars = Vars::from_json(state.outputs());
                Ok(Response::new(ActionState {
                    start_time: state.start_time,
                    end_time: state.end_time,
                    data: Some(vars.prost_vars()),
                }))
            }
            Err(err) => Err(Status::new(Code::Internal, err.to_string())),
        }
    };
}

macro_rules! wrap_result {
    ($input: expr) => {
        match $input {
            Ok(state) => Ok(Response::new(state)),
            Err(err) => Err(Status::new(Code::Internal, err.to_string())),
        }
    };
}

#[derive(Clone)]
pub struct MessageClient {
    client_id: String,
    addr: String,
    sender: Sender<Result<WorkflowMessage, Status>>,
    match_options: MatchOptions,
}

#[derive(Clone)]
pub struct MatchOptions {
    kind: String,
    event: String,
    nkind: String,
    topic: String,
}

impl MessageClient {
    fn matches(&self, message: &WorkflowMessage) -> std::result::Result<bool, globset::Error> {
        let pat_kind = globset::Glob::new(&self.match_options.kind)?.compile_matcher();
        let pat_event = globset::Glob::new(&self.match_options.event)?.compile_matcher();
        let pat_nkind = globset::Glob::new(&self.match_options.nkind)?.compile_matcher();
        let pat_mname = globset::Glob::new(&self.match_options.topic)?.compile_matcher();
        Ok(pat_kind.is_match(&message.kind)
            && pat_event.is_match(&message.event)
            && pat_nkind.is_match(&message.nkind)
            && pat_mname.is_match(&message.topic))
    }
}

#[derive(Clone)]
pub struct GrpcServer {
    engine: Arc<Engine>,
    clients: Arc<Mutex<HashMap<String, MessageClient>>>,
}

impl GrpcServer {
    pub fn new(engine: &Arc<Engine>) -> Self {
        let inst = Self {
            engine: engine.clone(),
            clients: Arc::new(Mutex::new(HashMap::new())),
        };

        inst
    }

    pub fn do_action(&self, name: &str, options: &Vars) -> Result<Response<ActionState>, Status> {
        tracing::info!("do-action  name={name} options={options}");
        let executor = self.engine.executor();
        match name {
            "ack" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let aid = options
                    .value_str("aid")
                    .ok_or(Status::invalid_argument("aid is required"))?;

                wrap_state_result!(executor.ack(pid, aid))
            }
            "submit" => {
                let mid = options
                    .value_str("mid")
                    .ok_or(Status::invalid_argument("mid is required"))?;
                wrap_state_result!(executor.submit(mid, &options.json_vars()))
            }
            "complete" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let aid = options
                    .value_str("aid")
                    .ok_or(Status::invalid_argument("aid is required"))?;

                wrap_state_result!(executor.complete(pid, aid, &options.json_vars()))
            }
            "abort" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let aid = options
                    .value_str("aid")
                    .ok_or(Status::invalid_argument("aid is required"))?;

                wrap_state_result!(executor.abort(pid, aid, &options.json_vars()))
            }
            "cancel" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let aid = options
                    .value_str("aid")
                    .ok_or(Status::invalid_argument("aid is required"))?;

                wrap_state_result!(executor.cancel(pid, aid, &options.json_vars()))
            }
            "back" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let aid = options
                    .value_str("aid")
                    .ok_or(Status::invalid_argument("aid is required"))?;

                wrap_state_result!(executor.back(pid, aid, &options.json_vars()))
            }
            "update" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let aid = options
                    .value_str("aid")
                    .ok_or(Status::invalid_argument("aid is required"))?;

                wrap_state_result!(executor.back(pid, aid, &options.json_vars()))
            }
            "models" => {
                let count = options.value_number("count").map_or(100, |v| v as usize);
                let manager = self.engine.manager();
                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let ret = manager.models(count).map(|data| {
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                    let mut arr: Vec<ActValue> = Vec::new();
                    for info in data {
                        arr.push(info.into());
                    }

                    let mut vars = Vars::new();
                    vars.insert(name.to_string(), &ActValue::Array(arr));
                    ActionState {
                        start_time: start_time.as_millis() as i64,
                        end_time: end_time.as_millis() as i64,
                        data: Some(vars.prost_vars()),
                    }
                });
                wrap_result!(ret)
            }
            "rm" => {
                let model_id = options
                    .value_str("mid")
                    .ok_or(Status::invalid_argument("mid is required"))?;
                let manager = self.engine.manager();
                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let ret = manager.remove(model_id).map(|data| {
                    let mut vars = Vars::new();
                    vars.insert(name.to_string(), &data.into());
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    ActionState {
                        start_time: start_time.as_millis() as i64,
                        end_time: end_time.as_millis() as i64,
                        data: Some(vars.prost_vars()),
                    }
                });
                wrap_result!(ret)
            }

            "model" => {
                let mid = options
                    .value_str("mid")
                    .ok_or(Status::invalid_argument("mid is required"))?;
                let manager = self.engine.manager();
                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let ret = manager.model(mid).map(|data| {
                    let mut vars = Vars::new();
                    vars.insert(name.to_string(), &data.into());
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    ActionState {
                        start_time: start_time.as_millis() as i64,
                        end_time: end_time.as_millis() as i64,
                        data: Some(vars.prost_vars()),
                    }
                });
                wrap_result!(ret)
            }

            "deploy" => {
                let model_text = options
                    .value_str("model")
                    .ok_or(Status::invalid_argument("model is required"))?;

                let model =
                    Workflow::from_str(model_text).map_err(|err| Status::invalid_argument(err))?;
                wrap_state_result!(self.engine.manager().deploy(&model))
            }

            "procs" => {
                let count = options.value_number("count").map_or(100, |v| v as usize);
                let manager = self.engine.manager();
                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let ret = manager.procs(count).map(|data| {
                    tracing::info!("procs={:?}", data);
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    let mut arr: Vec<ActValue> = Vec::new();
                    for info in data {
                        arr.push(info.into());
                    }

                    let mut vars = Vars::new();
                    vars.insert(name.to_string(), &ActValue::Array(arr));
                    ActionState {
                        start_time: start_time.as_millis() as i64,
                        end_time: end_time.as_millis() as i64,
                        data: Some(vars.prost_vars()),
                    }
                });
                wrap_result!(ret)
            }

            "proc" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let manager = self.engine.manager();
                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let ret = manager.proc(pid).map(|data| {
                    let mut vars = Vars::new();
                    vars.insert(name.to_string(), &data.into());
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    ActionState {
                        start_time: start_time.as_millis() as i64,
                        end_time: end_time.as_millis() as i64,
                        data: Some(vars.prost_vars()),
                    }
                });
                wrap_result!(ret)
            }

            "tasks" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let manager = self.engine.manager();

                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let ret = manager.tasks(pid).map(|data| {
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                    let mut arr: Vec<ActValue> = Vec::new();
                    for info in data {
                        arr.push(info.into());
                    }

                    let mut vars = Vars::new();
                    vars.insert(name.to_string(), &ActValue::Array(arr));
                    ActionState {
                        start_time: start_time.as_millis() as i64,
                        end_time: end_time.as_millis() as i64,
                        data: Some(vars.prost_vars()),
                    }
                });
                wrap_result!(ret)
            }
            "task" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;
                let manager = self.engine.manager();

                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let ret = manager.task(pid, tid).map(|data| {
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    let mut vars = Vars::new();
                    vars.insert(name.to_string(), &data.into());
                    ActionState {
                        start_time: start_time.as_millis() as i64,
                        end_time: end_time.as_millis() as i64,
                        data: Some(vars.prost_vars()),
                    }
                });
                wrap_result!(ret)
            }
            "acts" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options.value_str("tid");
                let manager = self.engine.manager();
                let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let ret = manager.acts(pid, tid).map(|data| {
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    let mut arr: Vec<ActValue> = Vec::new();
                    for info in data {
                        arr.push(info.into());
                    }

                    let mut vars = Vars::new();
                    vars.insert(name.to_string(), &ActValue::Array(arr));
                    ActionState {
                        start_time: start_time.as_millis() as i64,
                        end_time: end_time.as_millis() as i64,
                        data: Some(vars.prost_vars()),
                    }
                });
                wrap_result!(ret)
            }
            "start" => {
                let mid = options
                    .value_str("mid")
                    .ok_or(Status::invalid_argument("mid is required"))?;

                wrap_state_result!(executor.start(mid, &options.json_vars()))
            }
            _ => Err(Status::not_found(format!("not found action '{name}'"))),
        }
    }

    pub fn init(&self) {
        let emitter = self.engine.emitter();
        let grpc = self.clone();
        emitter.on_message(move |msg: &Message| {
            let vars = Vars::from_json(&msg.vars);
            let message = WorkflowMessage {
                kind: msg.kind.to_string(),
                event: msg.event.to_string(),
                pid: msg.pid.to_string(),
                key: msg.key.clone(),
                vars: Some(vars.prost_vars()),
                mid: msg.mid.clone(),
                topic: msg.topic.clone(),
                nid: msg.nid.clone(),
                nkind: msg.nkind.clone(),
                tid: msg.tid.clone(),
            };

            let grpc = grpc.clone();
            tokio::spawn(async move {
                grpc.send_message(&message).await;
            });
        });
    }

    pub async fn send_message(&self, message: &WorkflowMessage) {
        let clients = self.clients.lock().await;
        for (_, client) in clients.iter() {
            match client.matches(message) {
                Ok(is_match) => {
                    if !is_match {
                        continue;
                    }
                }
                Err(err) => {
                    println!("[Error matches] client={} {}", client.addr, err);
                    continue;
                }
            }
            let msg = Ok(message.clone());
            match client.sender.send(msg).await {
                Ok(_) => {
                    println!("[OK] send to {}", client.client_id);
                }
                Err(err) => {
                    println!("[ERROR] send to {}, error={:?}", client.client_id, err);
                    // clients.remove(index);
                }
            }
        }
    }
}

#[tonic::async_trait]
impl ActsService for GrpcServer {
    type OnMessageStream = MessageStream;

    async fn on_message(
        &self,
        req: Request<MessageOptions>,
    ) -> Result<Response<Self::OnMessageStream>, Status> {
        let (tx, rx) = mpsc::channel::<Result<WorkflowMessage, Status>>(128);
        let mut clients = self.clients.lock().await;

        let addr = req.remote_addr().unwrap();
        let options = req.into_inner();

        tracing::info!("on_message: options={:?}", options);
        if clients.contains_key(&options.client_id) {
            clients.remove(&options.client_id);
        }
        let client = MessageClient {
            client_id: options.client_id.clone(),
            addr: addr.to_string(),
            sender: tx,
            match_options: MatchOptions {
                kind: options.kind.clone(),
                event: options.event.clone(),
                nkind: options.nkind.clone(),
                topic: options.topic.clone(),
            },
        };
        clients
            .entry(options.client_id)
            .and_modify(|entry| *entry = client.clone())
            .or_insert(client.clone());

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream) as Self::OnMessageStream))
    }

    async fn action(&self, req: Request<ActionOptions>) -> Result<Response<ActionState>, Status> {
        let cmd = req.into_inner();
        let name = cmd.name;
        let vars = Vars::from_prost(
            &cmd.options
                .ok_or(Status::invalid_argument("{name} <options> is required"))?,
        );
        self.do_action(&name, &vars)
    }
}

pub async fn start(
    addr: SocketAddr,
    opt: &acts::Options,
) -> Result<(), Box<dyn std::error::Error>> {
    init_log(opt);

    let engine = Arc::new(Engine::new_with_options(opt));
    engine.start();
    let server = GrpcServer::new(&engine);
    server.init();
    let grpc = ActsServiceServer::new(server);

    Server::builder().add_service(grpc).serve(addr).await?;

    Ok(())
}

fn init_log(opt: &acts::Options) {
    let file_appender = tracing_appender::rolling::hourly(&opt.log_dir, "acts.log");
    let timer = LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:9]"
    ));
    std::env::set_var(ACTS_ENV_LOG, &opt.log_level);
    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_env_filter(EnvFilter::from_env(ACTS_ENV_LOG))
        .with_writer(std::io::stdout.and(file_appender))
        .with_ansi(false)
        .init();
}
