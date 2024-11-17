use super::CommandRunner as Command;
use crate::util;
use acts_channel::{model::ModelInfo, Vars};
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
        #[arg(short, long, help = "expect to load the max count")]
        count: Option<u32>,
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
        ModelCommands::Ls { count } => ls(parent, count).await,
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

async fn ls(parent: &mut Command<'_>, count: &Option<u32>) -> Result<String, String> {
    let mut result = String::new();
    let mut options = Vars::new();
    if let Some(count) = count {
        options.set("count", count);
    };

    let resp = parent
        .client
        .send::<Vec<ModelInfo>>("model:ls", options)
        .await
        .map_err(|err| err.message().to_string())?;
    let models = resp.data.unwrap();

    let mut table = Table::new();
    table.add_row(row!["id", "name", "version", "size", "time"]);
    for m in models {
        table.add_row(row![
            m.id,
            m.name,
            format!("{}", m.ver),
            util::size(m.size),
            util::local_time(m.time)
        ]);
    }
    table.printstd();

    // print the elapsed
    let cost = resp.end_time - resp.start_time;
    result.push_str(&format!("(elapsed {cost}ms)"));

    Ok(result)
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
