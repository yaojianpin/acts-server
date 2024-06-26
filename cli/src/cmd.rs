use crate::{help, util};
use acts_channel::{self, ActsChannel, ActsOptions, Vars};
use serde_json::json;
use tonic::Status;

pub struct Command<'a> {
    env: Vars,
    client: &'a mut ActsChannel,
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
    pub fn new(client: &'a mut ActsChannel) -> Self {
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
                        ret = match self.env.get(&key.to_string()) {
                            Some(v) => v.to_string(),
                            None => "(nil)".to_string(),
                        };
                    }
                    "set" => {
                        let key = args.get(1).ok_or(Status::invalid_argument(help_text))?;
                        let last = args.last().unwrap();
                        let mut vtype = EnvValueType::Json;
                        let mut end_index = args.len() - 1;
                        if vec!["json", "int", "float", "string"].contains(last) {
                            end_index = args.len() - 2;
                            vtype = EnvValueType::parse(last)
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
                        self.env.insert(key, &self.to_json(&value, &vtype)?);

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
                let target_name = args.get(0).ok_or(Status::invalid_argument(help_text))?;
                let id = args.get(1).ok_or(Status::invalid_argument(help_text))?;

                let mut options = Vars::new();
                options.insert_str("name".to_string(), target_name.to_string());
                options.insert_str("id".to_string(), id.to_string());
                let resp = self.client.rm(&options).await?;
                ret = util::process_result(name, resp);
            }

            "sub" => {
                let client_id = args
                    .get(0)
                    .cloned()
                    .ok_or(Status::invalid_argument(help_text))?;

                // * means to sub all messages
                let r#type = args.get(1).cloned().unwrap_or("*");
                let state = args.get(2).cloned().unwrap_or("*");
                let tag = args.get(3).cloned().unwrap_or("*");
                let key = args.get(4).cloned().unwrap_or("*");
                self.client
                    .sub(
                        client_id,
                        |m| {
                            println!("[message]: {}", serde_json::to_string(&m).unwrap());
                        },
                        &ActsOptions {
                            r#type: Some(r#type.to_string()),
                            state: Some(state.to_string()),
                            tag: Some(tag.to_string()),
                            key: Some(key.to_string()),
                        },
                    )
                    .await;
            }
            "deploy" => {
                let file_path = args.first().ok_or(Status::invalid_argument(help_text))?;
                let text = std::fs::read_to_string(file_path)
                    .map_err(|err| Status::invalid_argument(err.to_string()))?;

                let mut options = Vars::new();
                options.insert_str("model".to_string(), text);

                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
            }
            "start" => {
                let mid = args.get(0).ok_or(Status::invalid_argument(help_text))?;
                let mut options = Vars::new();
                options.insert_str("mid".to_string(), mid.to_string());
                options.extend(&self.env);

                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
            }
            "submit" | "back" | "cancel" | "abort" | "complete" | "skip" | "error" | "push"
            | "remove" => {
                let pid = args.get(0).ok_or(Status::invalid_argument(help_text))?;
                let tid = args.get(1).ok_or(Status::invalid_argument(help_text))?;
                let mut options = Vars::new();
                options.insert_str("pid".to_string(), pid.to_string());
                options.insert_str("tid".to_string(), tid.to_string());
                options.extend(&self.env);

                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
            }
            "resend" => {
                let options = Vars::new();
                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
            }
            "models" | "procs" | "packages" => {
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
                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
            }

            "model" => {
                let mut options = Vars::new();
                options.insert_str(
                    "mid".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );

                if let Some(fmt) = args.get(1) {
                    options.insert_str("fmt".to_string(), fmt.to_string());
                };

                options.extend(&self.env);
                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
            }

            "proc" => {
                let mut options = Vars::new();
                options.insert_str(
                    "pid".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );
                if let Some(fmt) = args.get(1) {
                    options.insert_str("fmt".to_string(), fmt.to_string());
                };
                options.extend(&self.env);
                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
            }
            "tasks" | "messages" => {
                let mut options = Vars::new();
                options.insert_str(
                    "pid".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );
                if let Some(count) = args.get(1) {
                    options.insert_number(
                        "count".to_string(),
                        count
                            .parse::<f64>()
                            .map_err(|err| Status::invalid_argument(err.to_string()))?,
                    );
                };
                options.extend(&self.env);
                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
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
                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
            }
            "message" => {
                let mut options = Vars::new();
                options.insert_str(
                    "id".to_string(),
                    args.get(0)
                        .cloned()
                        .ok_or(Status::invalid_argument(help_text))?,
                );
                options.extend(&self.env);
                let resp = self.client.do_action(name, &options).await?;
                ret = util::process_result(name, resp);
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
