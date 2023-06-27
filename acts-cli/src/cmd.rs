use crate::{help, util};
use acts_grpc::{
    self, acts_service_client::ActsServiceClient, model::Message, ActionOptions, MessageOptions,
    Vars,
};
use futures::StreamExt;
use serde_json::json;
use tonic::{transport::Channel, Request, Status};

pub struct Command<'a> {
    env: Vars,
    client: &'a mut ActsServiceClient<Channel>,
}

enum EnvValueType {
    Int,
    Float,
    String,
    Json,
}

impl EnvValueType {
    fn parse(t: &str) -> Option<EnvValueType> {
        match t {
            "int" => Some(EnvValueType::Int),
            "float" => Some(EnvValueType::Float),
            "json" => Some(EnvValueType::Json),
            "string" => Some(EnvValueType::String),
            _ => None,
        }
    }
}

impl<'a> Command<'a> {
    pub fn new(client: &'a mut ActsServiceClient<Channel>) -> Self {
        Self {
            client,
            env: Vars::new(),
        }
    }

    pub async fn send(&mut self, name: &str, args: &[&str]) -> Result<String, Status> {
        let mut ret = name.to_string();

        let help_text = help::cmd(name);
        match name {
            "env" => {
                let cmd = args
                    .get(0)
                    .cloned()
                    .ok_or(Status::invalid_argument(help_text))?;

                match cmd {
                    "ls" => {
                        ret.clear();
                        for (k, v) in self.env.iter() {
                            ret.push_str(&format!("{k}: {}\n", v.to_string()));
                        }
                    }
                    "json" => {
                        ret.push_str(&self.env.to_string());
                    }
                    "get" => {
                        let key = args.get(1).ok_or(Status::invalid_argument(help_text))?;
                        ret = match self.env.get(key.clone()) {
                            Some(v) => v.to_string(),
                            None => "(nil)".to_string(),
                        };
                    }
                    "set" => {
                        let key = args.get(1).ok_or(Status::invalid_argument(help_text))?;
                        let last = args.last().unwrap();
                        let mut vtype = EnvValueType::String;
                        let mut end_index = args.len() - 1;
                        if vec!["json", "int", "float", "string"].contains(last) {
                            end_index = args.len() - 2;
                            vtype = EnvValueType::parse(last.clone())
                                .ok_or(Status::invalid_argument(help_text))?;
                        }
                        let mut value = String::new();
                        let mut start_index = 2;

                        while start_index <= end_index {
                            let v =
                                args.get(start_index)
                                    .cloned()
                                    .ok_or(Status::invalid_argument(format!(
                                        "{name} [key] [value] [type]"
                                    )))?;
                            value.push_str(v);
                            if start_index != end_index {
                                value.push_str(" ");
                            }
                            start_index += 1;
                        }
                        self.env
                            .insert(key.to_string(), &self.to_json(&value, &vtype)?);

                        ret = format!("{key}:{value}");
                    }
                    "rm" => {
                        let key = args
                            .get(1)
                            .cloned()
                            .ok_or(Status::invalid_argument(help_text))?;
                        self.env.rm(key);
                    }
                    "clear" => {
                        self.env.clear();
                    }
                    _ => {
                        ret = help_text.to_string();
                    }
                };
            }
            "rm" => {
                let mut options = Vars::new();
                options.insert_str(
                    "mid".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );

                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;
                ret = util::process_result(name, resp.into_inner());
            }

            "sub" => {
                let client_id = args
                    .get(0)
                    .cloned()
                    .ok_or(Status::invalid_argument(help_text))?;

                // * means to sub all messages
                let kind = args.get(1).cloned().unwrap_or("*");
                let event = args.get(2).cloned().unwrap_or("*");
                let nkind = args.get(3).cloned().unwrap_or("*");
                let topic = args.get(4).cloned().unwrap_or("*");
                let request = tonic::Request::new(MessageOptions {
                    client_id: client_id.to_string(),
                    kind: kind.to_string(),
                    event: event.to_string(),
                    nkind: nkind.to_string(),
                    topic: topic.to_string(),
                });
                let mut stream = self.client.on_message(request).await.unwrap().into_inner();
                tokio::spawn(async move {
                    while let Some(item) = stream.next().await {
                        if !item.is_err() {
                            let m: Message = item.unwrap().into();
                            println!("[message]: {}", serde_json::to_string(&m).unwrap());
                        }
                    }
                });
            }
            "deploy" => {
                let file_path = args.first().ok_or(Status::invalid_argument(help_text))?;
                let text = std::fs::read_to_string(file_path)
                    .map_err(|err| Status::invalid_argument(err.to_string()))?;

                let mut options = Vars::new();
                options.insert_str("model".to_string(), text);

                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;
                ret = util::process_result(name, resp.into_inner());
            }
            "start" | "submit" => {
                let mid = args.get(0).ok_or(Status::invalid_argument(help_text))?;
                let mut options = Vars::new();
                options.insert_str("mid".to_string(), mid.clone());

                options.extend(&self.env);

                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;

                ret = util::process_result(name, resp.into_inner());
            }
            "ack" => {
                let pid = args.get(0).ok_or(Status::invalid_argument(help_text))?;
                let aid = args.get(1).ok_or(Status::invalid_argument(help_text))?;
                let mut options = Vars::new();
                options.insert_str("pid".to_string(), pid.clone());
                options.insert_str("aid".to_string(), aid.clone());
                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;
                ret = util::process_result(name, resp.into_inner());
            }
            "back" | "cancel" | "abort" | "complete" | "update" => {
                let pid = args.get(0).ok_or(Status::invalid_argument(help_text))?;
                let aid = args.get(1).ok_or(Status::invalid_argument(help_text))?;
                let mut options = Vars::new();
                options.insert_str("pid".to_string(), pid.clone());
                options.insert_str("aid".to_string(), aid.clone());

                options.extend(&self.env);
                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;

                ret = util::process_result(name, resp.into_inner());
            }

            "models" | "procs" => {
                let mut options = Vars::new();
                if let Some(count) = args.get(0) {
                    options.insert_number(
                        "count".to_string(),
                        count
                            .parse::<f64>()
                            .map_err(|err| Status::invalid_argument(err.to_string()))?,
                    );
                };

                options.extend(&self.env);
                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;

                ret = util::process_result(name, resp.into_inner());
            }

            "model" => {
                let mut options = Vars::new();
                options.insert_str(
                    "mid".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );

                options.extend(&self.env);
                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;

                ret = util::process_result(name, resp.into_inner());
            }

            "proc" | "tasks" => {
                let mut options = Vars::new();
                options.insert_str(
                    "pid".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );

                options.extend(&self.env);
                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;

                ret = util::process_result(name, resp.into_inner());
            }
            "acts" => {
                let mut options = Vars::new();
                options.insert_str(
                    "pid".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );

                if let Some(tid) = args.get(1) {
                    options.insert_str("tid".to_string(), tid.clone());
                }

                options.extend(&self.env);
                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;

                ret = util::process_result(name, resp.into_inner());
            }
            "task" => {
                let mut options = Vars::new();
                options.insert_str(
                    "pid".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );
                options.insert_str(
                    "tid".to_string(),
                    args.get(1)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );
                options.extend(&self.env);
                let resp = self
                    .client
                    .action(Request::new(ActionOptions {
                        name: name.to_string(),
                        options: Some(options.prost_vars()),
                    }))
                    .await?;

                ret = util::process_result(name, resp.into_inner());
            }
            "help" | _ => {
                ret = help::all();
            }
        }
        Ok(ret)
    }

    fn to_json(&self, value: &str, vtype: &EnvValueType) -> Result<serde_json::Value, Status> {
        match vtype {
            EnvValueType::Int => {
                let v = value
                    .parse::<i64>()
                    .map_err(|err| Status::invalid_argument(err.to_string()))?;
                Ok(json!(v))
            }
            EnvValueType::Float => {
                let v = value
                    .parse::<f64>()
                    .map_err(|err| Status::invalid_argument(err.to_string()))?;
                Ok(json!(v))
            }
            EnvValueType::String => Ok(json!(value)),
            EnvValueType::Json => {
                //
                Ok(serde_json::de::from_str::<serde_json::Value>(value)
                    .map_err(|err| Status::invalid_argument(err.to_string()))?)
            }
        }
    }
}
