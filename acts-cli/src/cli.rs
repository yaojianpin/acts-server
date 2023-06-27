use clap::{arg, command, Parser};

#[derive(Parser, Debug)]
#[command(name = "acts-cli", disable_help_flag = true)]
#[command(version = "1.0")]
#[command(about = "cli for acts-server", long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub hostname: Option<String>,

    #[arg(short, long)]
    pub port: Option<u16>,
    // #[arg(short = 'a', long)]
    // pub password: Option<String>,
}

impl Cli {}
