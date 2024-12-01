use super::CommandRunner as Command;
use crate::util;
use acts_channel::{
    model::{MessageInfo, PageData},
    ActsOptions, Vars,
};
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
        #[arg(short, long, help = "skip the offset number to begin count")]
        offset: Option<u32>,
        #[arg(short, long, help = "expect to load the item count")]
        count: Option<u32>,

        #[arg(short='Q', long, help = "query by keys. \nexample: -Q state=running -Q type=irq", value_parser = util::parse_key_value)]
        query_by: Vec<(String, String)>,

        #[arg(short='O', long, help = "order by keys. \nexample: -O state -O key,desc", value_parser = util::parse_sort)]
        order_by: Vec<(String, bool)>,
    },
    #[command(about = "remove a message by id")]
    Rm {
        #[arg(help = "message id")]
        id: String,
    },
    #[command(about = "clear all error messages")]
    Clear {
        #[arg(short, long, help = "proc id")]
        pid: Option<String>,
    },

    #[command(about = "redsend stored messages caused by error")]
    Redo,

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
    #[command(about = "unsubscribe server messages by client id")]
    Unsub {
        #[arg(help = "client id")]
        client_id: String,
    },
}

pub async fn process(parent: &mut Command<'_>, command: &MessageCommands) -> Result<(), String> {
    let ret = match command {
        MessageCommands::Get { id } => get(parent, id).await,
        MessageCommands::Ack { id } => ack(parent, id).await,
        MessageCommands::Ls {
            offset,
            count,
            query_by,
            order_by,
        } => ls(parent, offset, count, query_by, order_by).await,
        MessageCommands::Rm { id } => rm(parent, id).await,
        MessageCommands::Clear { pid } => clear(parent, pid).await,
        MessageCommands::Redo => redo(parent).await,
        MessageCommands::Sub {
            client_id,
            r#type,
            state,
            key,
            tag,
            ack,
        } => sub(parent, &client_id, r#type, state, key, tag, ack).await,
        MessageCommands::Unsub { client_id } => ubsub(parent, &client_id).await,
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

    // if let Some(pid) = pid {
    //     options.set("pid", pid);
    // };
    // if let Some(t) = r#type {
    //     options.set("type", t);
    // };
    // if let Some(key) = key {
    //     options.set("key", key);
    // };
    // if let Some(state) = state {
    //     options.set("state", state);
    // };
    // if let Some(status) = status {
    //     options.set("status", status);
    // };
    if let Some(offset) = offset {
        options.set("offset", offset);
    };
    if let Some(count) = count {
        options.set("count", count);
    };
    options.set("query_by", query_by);
    options.set("order_by", order_by);

    let resp = parent
        .client
        .send::<PageData<MessageInfo>>("msg:ls", options)
        .await
        .map_err(|err| err.message().to_string())?;

    let data = resp.data.as_ref().unwrap();
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
    for p in &data.rows {
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
    util::print_pager(&mut ret, &data);
    util::print_cost(&mut ret, &resp);

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

pub async fn clear(parent: &mut Command<'_>, pid: &Option<String>) -> Result<String, String> {
    let mut ret = String::new();
    let resp = parent
        .client
        .send::<bool>("msg:clear", Vars::new().with("pid", pid))
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

pub async fn ubsub(parent: &mut Command<'_>, client_id: &str) -> Result<String, String> {
    let mut ret = String::new();
    let resp = parent
        .client
        .send::<()>("msg:unsub", Vars::new().with("client_id", client_id))
        .await
        .map_err(|err| err.message().to_string())?;

    // print the elapsed
    let cost = resp.end_time - resp.start_time;
    ret.push_str(&format!("(elapsed {cost}ms)"));

    Ok(ret)
}
