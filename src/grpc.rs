use acts::{ActError, Builder, ChannelOptions, Engine, Message, Workflow};
use acts_channel::{
    acts_service_server::*, ActionOptions, MessageOptions, ProtoJsonValue, Vars, WorkflowMessage,
    WorkflowModel,
};
use futures::Stream;
use std::collections::HashMap;
use std::{net::SocketAddr, pin::Pin, sync::Arc};
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Code, Request, Response, Status};

type MessageStream = Pin<Box<dyn Stream<Item = Result<WorkflowMessage, Status>> + Send>>;

macro_rules! wrap_result {
    ($name:expr, $input: expr) => {
        match $input {
            Ok(data) => {
                let mut vars = Vars::new();
                vars.insert($name, &data.into());
                Ok(Response::new(vars.prost_vars()))
            }
            Err(err) => Err(Status::new(Code::Internal, err.to_string())),
        }
    };
}

#[derive(Clone)]
pub struct MessageClient {
    addr: String,
    sender: Sender<Result<WorkflowMessage, Status>>,
    options: ChannelOptions,
}

impl MessageClient {
    fn send(&self, msg: &Message) {
        let inputs = Vars::from_json(&msg.inputs);
        let outputs = Vars::from_json(&msg.outputs);
        let message = WorkflowMessage {
            id: msg.id.clone(),
            name: msg.name.clone(),
            source: msg.source.clone(),
            r#type: msg.r#type.clone(),
            model: Some(WorkflowModel {
                id: msg.model.id.clone(),
                name: msg.model.name.clone(),
                tag: msg.model.tag.clone(),
            }),
            key: msg.key.clone(),
            pid: msg.pid.clone(),
            tid: msg.tid.clone(),
            state: msg.state.to_string(),
            tag: msg.tag.clone(),
            start_time: msg.start_time,
            end_time: msg.end_time,

            inputs: Some(inputs.prost_vars()),
            outputs: Some(outputs.prost_vars()),
        };
        let msg = Ok(message.clone());
        let client = self.clone();
        tokio::spawn(async move {
            match client.sender.send(msg).await {
                Ok(_) => {
                    println!("[OK] send to {}({})", client.addr, client.options.id);
                }
                Err(err) => {
                    println!(
                        "[ERROR] send to {}({}), error={:?}",
                        client.addr, client.options.id, err
                    );
                    // clients.remove(index);
                }
            }
        });
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

    fn do_action(&self, name: &str, options: &Vars) -> Result<Response<ProtoJsonValue>, Status> {
        tracing::info!("do-action  name={name} options={options}");
        let executor = self.engine.executor();
        match name {
            "push" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;
                wrap_result!(name, executor.push(pid, tid, &options.json_vars().into()))
            }
            "remove" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(name, executor.remove(pid, tid, &options.json_vars().into()))
            }
            "submit" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(name, executor.submit(pid, tid, &options.json_vars().into()))
            }
            "complete" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(
                    name,
                    executor.complete(pid, tid, &options.json_vars().into())
                )
            }
            "abort" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(name, executor.abort(pid, tid, &options.json_vars().into()))
            }
            "cancel" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(name, executor.cancel(pid, tid, &options.json_vars().into()))
            }
            "back" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(name, executor.back(pid, tid, &options.json_vars().into()))
            }
            "skip" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(name, executor.skip(pid, tid, &options.json_vars().into()))
            }
            "error" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(name, executor.error(pid, tid, &options.json_vars().into()))
            }
            "models" => {
                let count = options.value_number("count").map_or(100, |v| v as usize);
                let manager = self.engine.manager();
                let ret = manager.models(count);
                wrap_result!(name, ret)
            }
            "rm" => {
                let target = options
                    .value_str("name")
                    .ok_or(Status::invalid_argument("name is required"))?;
                let id = options
                    .value_str("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                let manager = self.engine.manager();
                let ret = match target {
                    "model" => manager.rm_model(id),
                    "package" => manager.rm_package(id),
                    "message" => manager.rm_message(id),
                    _ => Err(ActError::Runtime(
                        "the name must be one of 'model', 'package' and 'message'".to_string(),
                    )),
                };
                wrap_result!(name, ret)
            }

            "resend" => {
                let manager = self.engine.manager();
                let ret = manager.resend_error_messages();
                wrap_result!(name, ret)
            }
            "model" => {
                let mid = options
                    .value_str("mid")
                    .ok_or(Status::invalid_argument("mid is required"))?;
                let fmt = options.value_str("fmt").unwrap_or("text");
                let manager = self.engine.manager();
                let ret = manager.model(mid, fmt);
                wrap_result!(name, ret)
            }
            "deploy" => {
                let model_text = options
                    .value_str("model")
                    .ok_or(Status::invalid_argument("model is required"))?;

                let mut model =
                    Workflow::from_yml(model_text).map_err(|err| Status::invalid_argument(err))?;
                if let Some(mid) = options.value_str("mid") {
                    model.set_id(mid);
                };
                wrap_result!(name, self.engine.manager().deploy(&model))
            }
            "procs" => {
                let count = options.value_number("count").map_or(100, |v| v as usize);
                let manager = self.engine.manager();
                let ret = manager.procs(count);
                wrap_result!(name, ret)
            }
            "proc" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let manager = self.engine.manager();
                let ret = manager.proc(pid);
                wrap_result!(name, ret)
            }
            "tasks" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let count = options.value_number("count").map_or(100, |v| v as usize);
                let manager = self.engine.manager();
                let ret = manager.tasks(pid, count);
                wrap_result!(name, ret)
            }
            "packages" => {
                let count = options.value_number("count").map_or(100, |v| v as usize);
                let manager = self.engine.manager();
                let ret = manager.packages(count);
                wrap_result!(name, ret)
            }
            "messages" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let count = options.value_number("count").map_or(100, |v| v as usize);
                let manager = self.engine.manager();
                let ret = manager.messages(pid, count);
                wrap_result!(name, ret)
            }
            "task" => {
                let pid = options
                    .value_str("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .value_str("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;
                let manager = self.engine.manager();
                let ret = manager.task(pid, tid);
                wrap_result!(name, ret)
            }
            "message" => {
                let id = options
                    .value_str("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                let manager = self.engine.manager();
                let ret = manager.message(id);
                wrap_result!(name, ret)
            }
            "start" => {
                let mid = options
                    .value_str("mid")
                    .ok_or(Status::invalid_argument("mid is required"))?;
                wrap_result!(name, executor.start(mid, &options.json_vars().into()))
            }
            "ack" => {
                let id = options
                    .value_str("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                wrap_result!(name, executor.ack(id))
            }
            _ => Err(Status::not_found(format!("not found action '{name}'"))),
        }
    }

    pub async fn init(&self) {
        let clients = self.clients.lock().await;
        for (_, client) in clients.iter() {
            let chan = self.engine.channel_with_options(&client.options);
            let c = client.clone();
            chan.on_message(move |e| {
                c.send(e);
            });
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
            addr: addr.to_string(),
            sender: tx,
            options: ChannelOptions {
                r#type: options.r#type.clone(),
                state: options.state.clone(),
                tag: options.tag.clone(),
                key: options.key.clone(),
                ack: true,
                id: options.client_id.clone(),
            },
        };
        clients
            .entry(options.client_id)
            .and_modify(|entry| *entry = client.clone())
            .or_insert(client.clone());

        let chan = self.engine.channel_with_options(&client.options);
        let c = client.clone();
        chan.on_message(move |e| {
            c.send(e);
        });

        let out_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(out_stream) as Self::OnMessageStream))
    }

    async fn action(
        &self,
        req: Request<ActionOptions>,
    ) -> Result<Response<ProtoJsonValue>, Status> {
        let cmd = req.into_inner();
        let name = cmd.name;
        let vars = Vars::from_prost(
            &cmd.options
                .ok_or(Status::invalid_argument("{name} <options> is required"))?,
        );
        self.do_action(&name, &vars)
    }
}

pub async fn start(addr: SocketAddr, opt: &acts::Config) -> Result<(), Box<dyn std::error::Error>> {
    init_log(opt);

    let mut builder = Builder::new();
    builder.set_config(&opt);
    let engine = Arc::new(builder.build());
    let server = GrpcServer::new(&engine);
    server.init().await;
    let grpc = ActsServiceServer::new(server);

    Server::builder().add_service(grpc).serve(addr).await?;

    Ok(())
}

fn init_log(#[allow(unused_variables)] opt: &acts::Config) {
    // disable the set_global_default error in tests
    #[cfg(not(test))]
    {
        use time::macros::format_description;
        use tracing_subscriber::fmt::time::LocalTime;
        use tracing_subscriber::fmt::writer::MakeWriterExt;
        use tracing_subscriber::EnvFilter;

        const ACTS_ENV_LOG: &str = "ACTS_LOG";

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
}
