mod cli;
mod client;
mod cmd;
mod help;
mod util;

use clap::Parser;
use cli::Cli;
use cmd::Command;
use std::io::{self, Write};
use std::str::FromStr;
use tonic::transport::Endpoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut port: u16 = 10080;
    let mut hostname = "127.0.0.1";

    if let Some(h) = &cli.hostname {
        hostname = h;
    }

    if let Some(p) = cli.port {
        port = p;
    }

    let uri = format!("http://{hostname}:{port}");
    let endpoint = Endpoint::from_str(&uri)?;
    let tip = format!("{}:{}> ", hostname, port);
    let mut client = client::connect(endpoint).await?;
    let mut cmd = Command::new(&mut client);

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    loop {
        print!("{}", tip);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).expect("Failed to read command");

        let commands: Vec<&str> = input.split_whitespace().collect();
        if commands.len() == 0 {
            continue;
        }
        let command: &str = &commands[0].to_lowercase();
        let args = &commands[1..];
        match cmd.send(&command, args).await {
            Ok(value) => {
                for line in value.lines() {
                    writeln!(stdout, "{}", line).unwrap();
                }
            }
            Err(err) => {
                let message = err.message();
                for line in message.lines() {
                    writeln!(stderr, "{}", line).unwrap();
                }
            }
        };
    }
}
