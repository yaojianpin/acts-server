use super::CommandRunner as Command;
use acts_channel::Vars;
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
