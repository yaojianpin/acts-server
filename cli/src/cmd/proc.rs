use super::CommandRunner as Command;
use crate::util;
use acts_channel::{model::ProcInfo, Vars};
use clap::{Args, Subcommand};
use prettytable::{row, Table};

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ProcArgs {
    #[command(subcommand)]
    pub command: ProcCommands,
}

#[derive(Debug, Subcommand)]
pub enum ProcCommands {
    #[command(about = "get proc by id")]
    Get {
        #[arg(help = "proc id")]
        id: String,
        #[arg(short, long, help = "format to print, the value should be one of json and tree", value_parser(["json", "tree"]))]
        fmt: Option<String>,
    },
    #[command(about = "list all procs")]
    Ls {
        #[arg(short, long, help = "expect to load the max count")]
        count: Option<u32>,
    },
    #[command(about = "deploy a workflow model")]
    Start {
        #[arg(required = true, help = "proc id")]
        id: String,
        #[arg(short, long, help = "specify a pid for proc")]
        pid: Option<String>,
    },
}

pub async fn process(parent: &mut Command<'_>, command: &ProcCommands) -> Result<(), String> {
    let ret = match command {
        ProcCommands::Get { id, fmt } => get(parent, id, fmt).await,
        ProcCommands::Ls { count } => ls(parent, count).await,
        ProcCommands::Start { id, pid } => start(parent, id, pid, &parent.vars.clone()).await,
    }?;

    parent.output(&ret);
    Ok(())
}

async fn start(
    parent: &mut Command<'_>,
    mid: &str,
    pid: &Option<String>,
    vars: &Vars,
) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new().extend(vars);
    if let Some(pid) = pid {
        options.set("pid", pid);
    }
    let resp = parent
        .client
        .start(mid, options)
        .await
        .map_err(|err| err.message().to_string())?;

    let pid = resp.data.unwrap();
    ret.push_str(&format!("pid={pid}"));
    // print the elapsed
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

pub async fn get(
    parent: &mut Command<'_>,
    pid: &str,
    fmt: &Option<String>,
) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new();
    options.set("pid", pid);

    if let Some(fmt) = fmt {
        options.set("fmt", fmt);
    };

    let resp = parent
        .client
        .send::<ProcInfo>("proc:get", options)
        .await
        .map_err(|err| err.message().to_string())?;

    let proc = resp.data.unwrap();
    ret.push_str(&serde_json::to_string_pretty(&proc).unwrap());
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

pub async fn ls(parent: &mut Command<'_>, count: &Option<u32>) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new();
    if let Some(count) = count {
        options.set("count", count);
    };
    let resp = parent
        .client
        .send::<Vec<ProcInfo>>("proc:ls", options)
        .await
        .map_err(|err| err.message().to_string())?;
    let procs = resp.data.unwrap();
    let mut table = Table::new();
    table.add_row(row!["pid", "name", "model id", "state", "start time"]);
    for p in procs {
        table.add_row(row![
            p.id,
            p.name,
            p.mid,
            p.state,
            util::local_time(p.start_time)
        ]);
    }
    table.printstd();
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}
