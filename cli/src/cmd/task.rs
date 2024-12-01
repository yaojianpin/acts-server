use super::CommandRunner as Command;
use crate::util;
use acts_channel::{
    model::{PageData, TaskInfo},
    Vars,
};
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
        #[arg(short, long, help = "skip the offset number to begin count")]
        offset: Option<u32>,
        #[arg(short, long, help = "expect to load the max count")]
        count: Option<u32>,
        #[arg(short='Q', long, help = "query by keys. \nexample: -Q state=running -Q type=irq", value_parser = util::parse_key_value)]
        query_by: Vec<(String, String)>,
        #[arg(short='O', long, help = "order by keys. \nexample: -O state -O type,desc", value_parser = util::parse_sort)]
        order_by: Vec<(String, bool)>,
    },
}

pub async fn process(parent: &mut Command<'_>, command: &TaskCommands) -> Result<(), String> {
    let ret = match command {
        TaskCommands::Get { pid, tid } => get(parent, pid, tid).await,
        TaskCommands::Ls {
            offset,
            count,
            query_by,
            order_by,
        } => ls(parent, offset, count, query_by, order_by).await,
    }?;

    parent.output(&ret);
    Ok(())
}

pub async fn ls(
    parent: &mut Command<'_>,
    offset: &Option<u32>,
    count: &Option<u32>,
    query_by: &Vec<(String, String)>,
    order_by: &Vec<(String, bool)>,
) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new();
    options.set("query_by", query_by);
    options.set("order_by", order_by);

    if let Some(offset) = offset {
        options.set("offset", offset);
    };

    if let Some(count) = count {
        options.set("count", count);
    };
    let resp = parent
        .client
        .send::<PageData<TaskInfo>>("task:ls", options)
        .await
        .map_err(|err| err.message().to_string())?;

    let data = resp.data.as_ref().unwrap();
    let mut table = Table::new();
    table.add_row(row![
        "type",
        "pid",
        "tid",
        "name",
        "nid",
        "state",
        "tag",
        "key",
        "start time",
        "end time"
    ]);
    for p in &data.rows {
        table.add_row(row![
            p.r#type,
            p.pid,
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
    util::print_pager(&mut ret, &data);
    util::print_cost(&mut ret, &resp);

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
