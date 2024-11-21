mod act;
mod model;
mod msg;
mod pack;
mod proc;
mod task;
mod vars;

use act::ActArgs;
use acts_channel::{self, ActsChannel, Vars};
use clap::{command, Parser, Subcommand};
use model::ModelArgs;
use msg::MessageArgs;
use owo_colors::OwoColorize;
use pack::PacakgeArgs;
use proc::ProcArgs;
use std::io::Write;
use task::TaskArgs;
use vars::VarsArgs;

#[derive(Debug, Parser)]
#[command(name = "act")]
#[command(multicall = true, styles=crate::util::CLAP_STYLING)]
pub struct ActsRootCommand {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "set options for command arguments")]
    Vars(VarsArgs),
    #[command(about = "execute model commands")]
    Model(ModelArgs),
    #[command(about = "execute package commands")]
    Package(PacakgeArgs),
    #[command(about = "execute proc commands")]
    Proc(ProcArgs),
    #[command(about = "execute task commands")]
    Task(TaskArgs),
    #[command(about = "execute message commands")]
    Message(MessageArgs),
    #[command(about = "execute act commands")]
    Act(ActArgs),
    #[command(about = "exit the cli")]
    Exit,
}

pub struct CommandRunner<'a> {
    vars: Vars,
    client: &'a mut ActsChannel,
}

impl<'a> CommandRunner<'a> {
    pub fn new(client: &'a mut ActsChannel) -> Self {
        Self {
            client,
            vars: Vars::new(),
        }
    }

    pub async fn run(&mut self, line: &str) -> Result<bool, String> {
        let args = shlex::split(line).ok_or("error: Invalid input")?;
        let cli = ActsRootCommand::try_parse_from(args).map_err(|e| {
            e.print().unwrap();
            "".to_string()
        })?;
        let command = cli.command;
        match command {
            Commands::Exit => {
                return Ok(true);
            }
            Commands::Vars(args) => {
                vars::process(self, &args.command).await?;
            }
            Commands::Model(args) => {
                model::process(self, &args.command).await?;
            }
            Commands::Package(args) => {
                pack::process(self, &args.command).await?;
            }
            Commands::Proc(args) => {
                proc::process(self, &args.command).await?;
            }
            Commands::Task(args) => {
                task::process(self, &args.command).await?;
            }
            Commands::Message(args) => {
                msg::process(self, &args.command).await?;
            }
            Commands::Act(args) => {
                act::process(self, &args.command).await?;
            }
        };

        Ok(false)
    }

    pub fn output(&self, value: &str) {
        for line in value.lines() {
            writeln!(std::io::stdout(), "{}", line.green()).unwrap();
        }
    }
}
