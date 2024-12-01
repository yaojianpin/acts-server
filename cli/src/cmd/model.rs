use super::CommandRunner as Command;
use crate::util;
use acts_channel::{
    model::{ModelInfo, PageData},
    Vars,
};
use clap::{Args, Subcommand};
use prettytable::{row, Table};
use std::path::PathBuf;

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ModelArgs {
    #[command(subcommand)]
    pub command: ModelCommands,
}

#[derive(Debug, Subcommand)]
pub enum ModelCommands {
    #[command(about = "get model by id")]
    Get {
        #[arg(help = "model id")]
        id: String,

        #[arg(short, long, help = "format to print, the value should be one of json and tree", value_parser(["json", "tree"]))]
        fmt: Option<String>,
    },

    #[command(about = "list all models")]
    Ls {
        #[arg(short, long, help = "skip the offset number to begin count")]
        offset: Option<u32>,
        #[arg(short, long, help = "expect to load the max count")]
        count: Option<u32>,
        #[arg(short='Q', long, help = "query by keys. \nexample: -Q name=approve", value_parser = util::parse_key_value)]
        query_by: Vec<(String, String)>,
        #[arg(short='O', long, help = "order by keys. \nexample: -O create_time -O update_time,desc", value_parser = util::parse_sort)]
        order_by: Vec<(String, bool)>,
    },

    #[command(about = "remove a model by id")]
    Rm {
        #[arg(help = "model id")]
        id: String,
    },

    #[command(about = "deploy a workflow model")]
    Deploy {
        #[arg(required = true, help = "model file path")]
        path: PathBuf,
    },
}

pub async fn process(parent: &mut Command<'_>, command: &ModelCommands) -> Result<(), String> {
    let ret = match command {
        ModelCommands::Get { id, fmt } => get(parent, id, fmt).await,
        ModelCommands::Ls {
            offset,
            count,
            query_by,
            order_by,
        } => ls(parent, offset, count, query_by, order_by).await,
        ModelCommands::Rm { id } => rm(parent, id).await,
        ModelCommands::Deploy { path } => deploy(parent, path).await,
    }?;

    parent.output(&ret);
    Ok(())
}

async fn deploy(parent: &mut Command<'_>, path: &PathBuf) -> Result<String, String> {
    let mut ret = String::new();
    let text = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    let resp = parent
        .client
        .deploy(&text, None)
        .await
        .map_err(|err| err.message().to_string())?;
    // print the elapsed
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

async fn ls(
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
        .send::<PageData<ModelInfo>>("model:ls", options)
        .await
        .map_err(|err| err.message().to_string())?;
    let data = resp.data.as_ref().unwrap();
    let mut table = Table::new();
    table.add_row(row![
        "id",
        "name",
        "version",
        "size",
        "create time",
        "update time"
    ]);
    for m in &data.rows {
        table.add_row(row![
            m.id,
            m.name,
            format!("{}", m.ver),
            util::size(m.size),
            util::local_time(m.create_time),
            util::local_time(m.update_time)
        ]);
    }

    table.printstd();
    util::print_pager(&mut ret, &data);
    util::print_cost(&mut ret, &resp);

    Ok(ret)
}

async fn get(parent: &mut Command<'_>, id: &str, fmt: &Option<String>) -> Result<String, String> {
    let mut ret = String::new();
    let mut options = Vars::new();
    options.set("id", id);
    if let Some(fmt) = fmt {
        options.set("fmt", fmt);
    };

    let resp = parent
        .client
        .send::<ModelInfo>("model:get", options)
        .await
        .map_err(|err| err.message().to_string())?;
    let model = resp.data.unwrap();
    ret.push_str(&model.data);
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}

async fn rm(parent: &mut Command<'_>, id: &str) -> Result<String, String> {
    let mut ret = String::new();
    let resp = parent
        .client
        .send::<bool>("model:rm", Vars::new().with("id", id))
        .await
        .map_err(|err| err.message().to_string())?;

    // print the elapsed
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}
