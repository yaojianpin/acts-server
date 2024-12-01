use crate::utils;
use acts::ExecutorQuery;
use acts::{data::Package, Builder, ChannelOptions, Engine, Workflow};
use acts_channel::MessageOptions;
use acts_channel::{acts_service_server::*, Message};
use std::{net::SocketAddr, pin::Pin, sync::Arc};
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Code, Response, Status};

type MessageStream = Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send>>;

macro_rules! wrap_result {
    ($seq: expr, $name:expr, $input: expr) => {
        match $input {
            Ok(data) => {
                let mut message = utils::wrap_message($name, &data);
                message.ack = Some($seq.to_string());
                Ok(Response::new(message))
            }
            Err(err) => {
                println!("wrap_result err= {err:?}");
                Err(Status::new(Code::Internal, err.to_string()))
            }
        }
    };
}

#[derive(Clone)]
pub struct MessageClient {
    addr: String,
    sender: Sender<Result<Message, Status>>,
    options: ChannelOptions,
}

impl MessageClient {
    fn send(&self, message: Message) {
        let msg = Ok(message);
        let client = self.clone();
        if client.sender.is_closed() {
            println!(
                "[ERROR] client {}({}) is closed",
                client.addr, client.options.id
            );
            return;
        }
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
}

impl GrpcServer {
    pub fn new(engine: &Arc<Engine>) -> Self {
        let inst = Self {
            engine: engine.clone(),
        };

        inst
    }

    fn do_action(&self, message: Message) -> Result<Response<Message>, Status> {
        let options = match message.data {
            Some(data) => &serde_json::from_slice::<acts::Vars>(&data).unwrap(),
            None => &acts::Vars::new(),
        };
        tracing::info!(
            "do-action seq={} name={} ack={:?} options={options}",
            message.seq,
            message.name,
            message.ack
        );

        let name = message.name.as_str();
        let ack = message.seq.as_str();
        let executor = self.engine.executor();
        match name {
            // do act
            "act:push" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;
                wrap_result!(ack, name, executor.act().push(&pid, &tid, options))
            }
            "act:remove" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(ack, name, executor.act().remove(&pid, &tid, options))
            }
            "act:submit" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(ack, name, executor.act().submit(&pid, &tid, options))
            }
            "act:complete" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(ack, name, executor.act().complete(&pid, &tid, options))
            }
            "act:abort" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(ack, name, executor.act().abort(&pid, &tid, options))
            }
            "act:cancel" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(ack, name, executor.act().cancel(&pid, &tid, options))
            }
            "act:back" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(ack, name, executor.act().back(&pid, &tid, options))
            }
            "act:skip" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(ack, name, executor.act().skip(&pid, &tid, options))
            }
            "act:error" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;

                wrap_result!(ack, name, executor.act().error(&pid, &tid, options))
            }
            // model
            "model:ls" => {
                let offset = options.get::<i64>("offset").map_or(0, |v| v as usize);
                let count = options.get::<i64>("count").map_or(100, |v| v as usize);
                let query_by = options
                    .get::<Vec<(String, String)>>("query_by")
                    .unwrap_or_default();
                let order_by = options
                    .get::<Vec<(String, bool)>>("order_by")
                    .unwrap_or_default();
                let query = ExecutorQuery {
                    offset,
                    count,
                    query_by,
                    order_by,
                    ..Default::default()
                };
                let ret = executor.model().list(&query);
                wrap_result!(ack, name, ret)
            }
            "model:rm" => {
                let id = options
                    .get::<String>("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                let ret = executor.model().rm(&id);
                wrap_result!(ack, name, ret)
            }
            "model:get" => {
                let mid = options
                    .get::<String>("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                let fmt = options.get::<String>("fmt").unwrap_or("text".to_string());
                let ret = executor.model().get(&mid, &fmt);
                wrap_result!(ack, name, ret)
            }
            "model:deploy" => {
                let model_text = options
                    .get::<String>("model")
                    .ok_or(Status::invalid_argument("model is required"))?;

                let mut model =
                    Workflow::from_yml(&model_text).map_err(|err| Status::invalid_argument(err))?;
                if let Some(mid) = options.get::<String>("mid") {
                    model.set_id(&mid);
                };
                wrap_result!(ack, name, executor.model().deploy(&model))
            }
            // package
            "pack:ls" => {
                let offset = options.get::<i64>("offset").map_or(0, |v| v as usize);
                let count = options.get::<i64>("count").map_or(100, |v| v as usize);
                let query_by = options
                    .get::<Vec<(String, String)>>("query_by")
                    .unwrap_or_default();
                let order_by = options
                    .get::<Vec<(String, bool)>>("order_by")
                    .unwrap_or_default();
                let query = ExecutorQuery {
                    offset,
                    count,
                    query_by,
                    order_by,
                    ..Default::default()
                };
                let ret = executor.pack().list(&query);
                wrap_result!(ack, name, ret)
            }
            "pack:publish" => {
                let package_id = options
                    .get::<String>("id")
                    .ok_or(Status::invalid_argument("package 'id' is required"))?;
                let package_name = options.get::<String>("name").unwrap_or_default();
                let data = options
                    .get::<String>("body")
                    .ok_or(Status::invalid_argument("package 'body' is required"))?;
                let pack = Package {
                    id: package_id,
                    name: package_name,
                    data: data.into_bytes(),
                    ..Default::default()
                };
                wrap_result!(ack, name, executor.pack().publish(&pack))
            }
            "pack:rm" => {
                let id = options
                    .get::<String>("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                let ret = executor.pack().rm(&id);
                wrap_result!(ack, name, ret)
            }
            // proc
            "proc:start" => {
                let id = options
                    .get::<String>("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                wrap_result!(ack, name, executor.proc().start(&id, options))
            }
            "proc:ls" => {
                let offset = options.get::<i64>("offset").map_or(0, |v| v as usize);
                let count = options.get::<i64>("count").map_or(100, |v| v as usize);
                let query_by = options
                    .get::<Vec<(String, String)>>("query_by")
                    .unwrap_or_default();
                let order_by = options
                    .get::<Vec<(String, bool)>>("order_by")
                    .unwrap_or_default();
                let query = ExecutorQuery {
                    offset,
                    count,
                    query_by,
                    order_by,
                    ..Default::default()
                }
                .with_offset(offset)
                .with_count(count);

                let ret = executor.proc().list(&query);
                wrap_result!(ack, name, ret)
            }
            "proc:get" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let ret = executor.proc().get(&pid);
                wrap_result!(ack, name, ret)
            }
            // task
            "task:ls" => {
                let offset = options.get::<i64>("offset").map_or(0, |v| v as usize);
                let count = options.get::<i64>("count").map_or(100, |v| v as usize);
                let query_by = options
                    .get::<Vec<(String, String)>>("query_by")
                    .unwrap_or_default();
                let order_by = options
                    .get::<Vec<(String, bool)>>("order_by")
                    .unwrap_or_default();
                let query = ExecutorQuery {
                    offset,
                    count,
                    query_by,
                    order_by,
                    ..Default::default()
                };

                let ret = executor.task().list(&query);
                wrap_result!(ack, name, ret)
            }
            "task:get" => {
                let pid = options
                    .get::<String>("pid")
                    .ok_or(Status::invalid_argument("pid is required"))?;
                let tid = options
                    .get::<String>("tid")
                    .ok_or(Status::invalid_argument("tid is required"))?;
                let ret = executor.task().get(&pid, &tid);
                wrap_result!(ack, name, ret)
            }
            // msg
            "msg:ls" => {
                let offset = options.get::<i64>("offset").map_or(0, |v| v as usize);
                let count = options.get::<i64>("count").map_or(100, |v| v as usize);
                let order_by = options
                    .get::<Vec<(String, bool)>>("order_by")
                    .unwrap_or_default();
                let query_by = options
                    .get::<Vec<(String, String)>>("query_by")
                    .unwrap_or_default();
                let query = ExecutorQuery {
                    offset,
                    count,
                    query_by,
                    order_by,
                    ..Default::default()
                };
                let ret = executor.msg().list(&query);
                wrap_result!(ack, name, ret)
            }
            "msg:get" => {
                let id = options
                    .get::<String>("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                let ret = executor.msg().get(&id);
                wrap_result!(ack, name, ret)
            }
            "msg:ack" => {
                let id = options
                    .get::<String>("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                wrap_result!(ack, name, executor.msg().ack(&id))
            }
            "msg:redo" => {
                let ret = executor.msg().redo();
                wrap_result!(ack, name, ret)
            }
            "msg:clear" => {
                let pid = options.get::<String>("pid");
                let ret = executor.msg().clear(pid);
                wrap_result!(ack, name, ret)
            }
            "msg:rm" => {
                let id = options
                    .get::<String>("id")
                    .ok_or(Status::invalid_argument("id is required"))?;
                let ret = executor.msg().rm(&id);
                wrap_result!(ack, name, ret)
            }
            "msg:unsub" => {
                let client_id = options
                    .get::<String>("client_id")
                    .ok_or(Status::invalid_argument("client id is required"))?;
                let ret = executor.msg().unsub(&client_id);
                wrap_result!(ack, name, ret)
            }
            _ => Err(Status::not_found(format!("not found action '{name}'"))),
        }
    }

    pub async fn init(&self) {}
}

#[tonic::async_trait]
impl ActsService for GrpcServer {
    type OnMessageStream = MessageStream;

    async fn on_message(
        &self,
        req: tonic::Request<MessageOptions>,
    ) -> Result<tonic::Response<Self::OnMessageStream>, tonic::Status> {
        let (tx, rx) = mpsc::channel::<Result<Message, Status>>(128);
        let addr = req.remote_addr().unwrap();
        let options = req.into_inner();

        tracing::info!("on_message: options={:?}", options);
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
        let chan = self.engine.channel_with_options(&client.options);
        tokio::spawn(async move {
            chan.on_message(move |e| {
                let message = Message {
                    name: e.name.clone(),
                    seq: e.id.clone(),
                    ack: None,
                    data: Some(serde_json::to_vec(e.inner()).unwrap()),
                };
                client.send(message);
            });
        });

        let chan_stream = Box::pin(ReceiverStream::new(rx));
        Ok(Response::new(chan_stream))
    }

    async fn send(
        &self,
        request: tonic::Request<Message>,
    ) -> Result<tonic::Response<Message>, tonic::Status> {
        self.do_action(request.into_inner())
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
