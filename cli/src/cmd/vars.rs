use super::CommandRunner as Command;
use acts_channel::model::ActValue;
use clap::{Args, Subcommand, ValueEnum};
use serde_json::json;

#[derive(Debug, Subcommand)]
pub enum VarsCommands {
    #[command(about = "set a record with key and value")]
    Set {
        #[arg(help = "vars key in string")]
        key: String,
        #[arg(
            help = "vars value to set, the default format is string, to change format by using -c argument"
        )]
        value: String,

        #[arg(
            short,
            long,
            default_value_t = VarFmt::String,
            value_enum,
            help="vars value format"
        )]
        fmt: VarFmt,
    },
    #[command(about = "get value by key")]
    Get { key: String },
    #[command(about = "list all items")]
    Ls,
    #[command(about = "remove a item by key")]
    Rm { key: String },
    #[command(about = "clear all options")]
    Clear,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct VarsArgs {
    #[command(subcommand)]
    pub command: VarsCommands,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum VarFmt {
    Int,
    Float,
    String,
    Json,
}

pub async fn process(parent: &mut Command<'_>, command: &VarsCommands) -> Result<(), String> {
    let ret = match command {
        VarsCommands::Set { key, value, fmt } => set(parent, &key, &value, fmt),
        VarsCommands::Get { key } => get(parent, key),
        VarsCommands::Ls => ls(parent),
        VarsCommands::Rm { key } => rm(parent, key),
        VarsCommands::Clear => clear(parent),
    }?;

    parent.output(&ret);
    Ok(())
}

fn set(parent: &mut Command<'_>, key: &str, value: &str, fmt: &VarFmt) -> Result<String, String> {
    let value = to_json(&value, &fmt)?;
    parent.vars.set(key, &value);

    Ok(format!("{key}:{value}"))
}

fn get(parent: &mut Command<'_>, key: &str) -> Result<String, String> {
    let ret = match parent.vars.get::<ActValue>(key) {
        Some(v) => v.to_string(),
        None => "(nil)".to_string(),
    };

    Ok(ret)
}

fn rm(parent: &mut Command<'_>, key: &str) -> Result<String, String> {
    parent.vars.remove(key);
    Ok("".to_string())
}

fn ls(parent: &mut Command<'_>) -> Result<String, String> {
    let mut ret = String::new();
    if parent.vars.is_empty() {
        return Ok("(nil)".to_string());
    }
    for (k, v) in parent.vars.iter() {
        ret.push_str(&format!("{k}: {}\n", v.to_string()));
    }

    Ok(ret)
}

fn clear(parent: &mut Command<'_>) -> Result<String, String> {
    let ret = String::new();
    parent.vars.clear();
    Ok(ret)
}

fn to_json(value: &str, fmt: &VarFmt) -> Result<serde_json::Value, String> {
    match fmt {
        VarFmt::Int => {
            let v = value.parse::<i64>().map_err(|err| err.to_string())?;
            Ok(json!(v))
        }
        VarFmt::Float => {
            let v = value.parse::<f64>().map_err(|err| err.to_string())?;
            Ok(json!(v))
        }
        VarFmt::String => Ok(json!(value)),
        VarFmt::Json => {
            Ok(serde_json::de::from_str::<serde_json::Value>(value)
                .map_err(|err| err.to_string())?)
        }
    }
}
