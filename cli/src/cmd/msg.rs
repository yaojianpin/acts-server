use super::CommandRunner as Command;
use crate::util;
use acts_channel::{model::MessageInfo, Vars};
use clap::{Args, Subcommand};
use prettytable::{row, Table};

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct MessageArgs {
    #[command(subcommand)]
    pub command: MessageCommands,
}

#[derive(Debug, Subcommand)]
pub enum MessageCommands {
    #[command(about = "get message by id")]
    Get {
        #[arg(help = "message id")]
        id: String,
    },
    #[command(about = "ack message by id")]
    Ack {
        #[arg(help = "message id")]
        id: String,
    },
    #[command(about = "list all messages")]
    Ls {
        #[arg(short, long, help = "proc id")]
        pid: Option<String>,
        #[arg(short, long, help = "expect to load the max count")]
        count: Option<u32>,
    },
    #[command(about = "remove a message by id")]
    Rm {
        #[arg(help = "message id")]
        id: String,
    },
    #[command(about = "redsend stored messages caused by error")]
    Redo,
}

pub async fn process(parent: &mut Command<'_>, command: &MessageCommands) -> Result<(), String> {
    let ret = match command {
        MessageCommands::Get { id } => get(parent, id).await,
        MessageCommands::Ack { id } => ack(parent, id).await,
        MessageCommands::Ls { pid, count } => ls(parent, pid, count).await,
        MessageCommands::Rm { id } => rm(parent, id).await,
        MessageCommands::Redo => redo(parent).await,
    }?;

    parent.output(&ret);
    Ok(())
}

pub async fn ls(
    parent: &mut Command<'_>,
    pid: &Option<String>,
    count: &Option<u32>,
) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new();

    if let Some(pid) = pid {
        options.set("pid", pid);
    };
    if let Some(count) = count {
        options.set("count", count);
    };
    let resp = parent
        .client
        .send::<Vec<MessageInfo>>("msg:ls", options)
        .await
        .map_err(|err| err.message().to_string())?;

    let messages = resp.data.unwrap();
    let mut table = Table::new();
    table.add_row(row![
        "type",
        "id",
        // "name",
        // "pid",
        "tid",
        "state",
        "key",
        // "tag",
        "retries",
        "status",
        // "inputs",
        // "outputs",
        "create time",
        "update time"
    ]);
    for p in messages {
        table.add_row(row![
            p.r#type,
            p.id,
            // p.name,
            // p.pid,
            p.tid,
            p.state,
            p.key,
            // p.tag,
            p.retry_times,
            p.status,
            // p.inputs,
            // p.outputs,
            util::local_time(p.create_time),
            util::local_time(p.update_time)
        ]);
    }
    table.printstd();
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

pub async fn get(parent: &mut Command<'_>, id: &str) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new();
    options.set("id", id);
    let resp = parent
        .client
        .send::<MessageInfo>("msg:get", options)
        .await
        .map_err(|err| err.message().to_string())?;
    let message = resp.data.unwrap();
    ret.push_str(&serde_json::to_string_pretty(&message).unwrap());
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

pub async fn ack(parent: &mut Command<'_>, id: &str) -> Result<String, String> {
    let mut ret = String::new();
    let resp = parent
        .client
        .ack(id)
        .await
        .map_err(|err| err.message().to_string())?;

    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

pub async fn redo(parent: &mut Command<'_>) -> Result<String, String> {
    let mut ret = String::new();
    let options = Vars::new();
    let resp = parent
        .client
        .send::<()>("msg:redo", options)
        .await
        .map_err(|err| err.message().to_string())?;

    // print the elapsed
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

pub async fn rm(parent: &mut Command<'_>, id: &str) -> Result<String, String> {
    let mut ret = String::new();
    let resp = parent
        .client
        .send::<bool>("msg:rm", Vars::new().with("id", id))
        .await
        .map_err(|err| err.message().to_string())?;

    // print the elapsed
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}
