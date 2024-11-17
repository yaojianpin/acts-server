use super::CommandRunner as Command;
use crate::util;
use acts_channel::{model::TaskInfo, Vars};
use clap::{Args, Subcommand};
use prettytable::{row, Table};

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub command: TaskCommands,
}

#[derive(Debug, Subcommand)]
pub enum TaskCommands {
    #[command(about = "get task by id")]
    Get {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(help = "task id")]
        tid: String,
    },
    #[command(about = "list all tasks")]
    Ls {
        #[arg(help = "proc id")]
        pid: String,
        #[arg(short, long, help = "expect to load the max count")]
        count: Option<u32>,
    },
}

pub async fn process(parent: &mut Command<'_>, command: &TaskCommands) -> Result<(), String> {
    let ret = match command {
        TaskCommands::Get { pid, tid } => get(parent, pid, tid).await,
        TaskCommands::Ls { pid, count } => ls(parent, pid, count).await,
    }?;

    parent.output(&ret);
    Ok(())
}

pub async fn ls(
    parent: &mut Command<'_>,
    pid: &str,
    count: &Option<u32>,
) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new().with("pid", pid);
    if let Some(count) = count {
        options.set("count", count);
    };
    let resp = parent
        .client
        .send::<Vec<TaskInfo>>("task:ls", options)
        .await
        .map_err(|err| err.message().to_string())?;

    let tasks = resp.data.unwrap();
    let mut table = Table::new();
    table.add_row(row![
        "type",
        "tid",
        "name",
        "nid",
        "state",
        "tag",
        "key",
        "start time",
        "end time"
    ]);
    for p in tasks {
        table.add_row(row![
            p.r#type,
            p.id,
            p.name,
            p.nid,
            p.state,
            p.tag,
            p.key,
            util::local_time(p.start_time),
            util::local_time(p.end_time)
        ]);
    }
    table.printstd();
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

pub async fn get(parent: &mut Command<'_>, pid: &str, tid: &str) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new();
    options.set("pid", pid);
    options.set("tid", tid);
    let resp = parent
        .client
        .send::<TaskInfo>("task:get", options)
        .await
        .map_err(|err| err.message().to_string())?;
    let task = resp.data.unwrap();
    ret.push_str(&serde_json::to_string_pretty(&task).unwrap());
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}
