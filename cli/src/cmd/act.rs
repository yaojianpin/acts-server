use super::CommandRunner as Command;
use acts_channel::{ActsOptions, Vars};
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ActArgs {
    #[command(subcommand)]
    pub command: ActCommands,
}

#[derive(Debug, Subcommand)]
pub enum ActCommands {
    #[command(about = "submit a running act")]
    Submit {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
    },

    #[command(about = "complete a running act")]
    Complete {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
    },

    #[command(about = "skip a running act")]
    Skip {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
    },

    #[command(about = "abort a running act")]
    Abort {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
    },

    #[command(about = "set a running act as error")]
    Error {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
        #[arg(help = "error code")]
        ecode: String,
        #[arg(help = "error message")]
        error: Option<String>,
    },

    #[command(about = "cancel a completed act")]
    Cancel {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
    },

    #[command(about = "back a running act to a historical step")]
    Back {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
        #[arg(required = true, help = "model file path")]
        to: String,
    },

    #[command(about = "push a new act under a step")]
    Push {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
    },

    #[command(about = "remove an act from a step")]
    Remove {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
    },
    #[command(about = "subscribe server messages")]
    Sub {
        #[arg(help = "client id")]
        client_id: String,
        #[arg(
            short,
            long,
            help = "message type in glob pattern, the type includes workflow, step, branch and act"
        )]
        r#type: Option<String>,
        #[arg(
            short,
            long,
            help = "message type in glob pattern, the state includes created, completed, error, cancelled, aborted, skipped and backed"
        )]
        state: Option<String>,
        #[arg(
            long,
            help = "glob pattern for message tag which is defined in workflow tag attribute"
        )]
        tag: Option<String>,
        #[arg(short, long, help = "message key in glob pattern")]
        key: Option<String>,
        #[arg(
            short,
            long,
            default_value_t = true,
            help = "auto ack message by client, if false you should ack message from you app"
        )]
        ack: bool,
    },
}

pub async fn process(parent: &mut Command<'_>, command: &ActCommands) -> Result<(), String> {
    let ret = match command {
        ActCommands::Submit { pid, tid } => {
            send(parent, "act:submit", pid, tid, parent.vars.clone()).await
        }
        ActCommands::Complete { pid, tid } => {
            send(parent, "act:complete", pid, tid, parent.vars.clone()).await
        }
        ActCommands::Skip { pid, tid } => {
            send(parent, "act:skip", pid, tid, parent.vars.clone()).await
        }
        ActCommands::Abort { pid, tid } => {
            send(parent, "act:abort", pid, tid, parent.vars.clone()).await
        }
        ActCommands::Error {
            pid,
            tid,
            ecode,
            error,
        } => {
            let mut vars = Vars::new().with("ecode", ecode);
            if let Some(error) = error {
                vars.set("error", error);
            }
            let ret = send(parent, "act:error", pid, tid, vars.extend(&parent.vars)).await?;
            Ok(ret)
        }
        ActCommands::Cancel { pid, tid } => {
            send(parent, "act:cancel", pid, tid, parent.vars.clone()).await
        }
        ActCommands::Back { pid, tid, to } => {
            let vars = Vars::new().with("to", to);
            send(parent, "act:back", pid, tid, vars.extend(&parent.vars)).await
        }
        ActCommands::Push { pid, tid } => send(parent, "push", pid, tid, parent.vars.clone()).await,
        ActCommands::Remove { pid, tid } => {
            send(parent, "act:remove", pid, tid, parent.vars.clone()).await
        }
        ActCommands::Sub {
            client_id,
            r#type,
            state,
            key,
            tag,
            ack,
        } => sub(parent, &client_id, r#type, state, key, tag, ack).await,
    }?;

    parent.output(&ret);
    Ok(())
}

async fn send(
    parent: &mut Command<'_>,
    name: &str,
    pid: &str,
    tid: &str,
    vars: Vars,
) -> Result<String, String> {
    let mut ret = String::new();
    let options = Vars::new().with("pid", pid).with("tid", tid).extend(&vars);
    let resp = parent
        .client
        .send::<()>(name, options)
        .await
        .map_err(|err| err.message().to_string())?;

    // print the elapsed
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

async fn sub(
    parent: &mut Command<'_>,
    client_id: &str,
    r#type: &Option<String>,
    state: &Option<String>,
    tag: &Option<String>,
    key: &Option<String>,
    ack: &bool,
) -> Result<String, String> {
    let ret = String::new();

    let default_value = "*".to_string();
    // * means to sub all messages
    let r#type = r#type.as_ref().unwrap_or(&default_value);
    let state = state.as_ref().unwrap_or(&default_value);
    let tag = tag.as_ref().unwrap_or(&default_value);
    let key = key.as_ref().unwrap_or(&default_value);
    parent
        .client
        .subscribe(
            client_id,
            |m| {
                println!("[message]: {}", serde_json::to_string(&m).unwrap());
            },
            &ActsOptions {
                r#type: Some(r#type.to_string()),
                state: Some(state.to_string()),
                tag: Some(tag.to_string()),
                key: Some(key.to_string()),
                ack: Some(ack.clone()),
            },
        )
        .await;

    Ok(ret)
}
