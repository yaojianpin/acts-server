use acts::Config;
use std::{fs, path::Path};

mod config;
mod grpc;
#[cfg(test)]
mod tests;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut port = 10080;
    let mut options = Config::default();
    if let Ok(conf_file) = fs::read_to_string(&Path::new("acts.conf")) {
        if let Ok(conf) = hocon::de::from_str::<config::Config>(&conf_file) {
            port = conf.port.unwrap_or(10080);
            options.data_dir = conf.data_dir.unwrap_or("data".to_string());

            if let Some(log) = conf.log {
                options.log_dir = log.dir.unwrap_or("log".to_string());
                options.log_level = log.level.unwrap_or("INFO".to_string());
            }
        }
    }

    print_logo();
    println!(
        "The server is now ready to accept connections on port {}",
        port
    );

    let addr = format!("0.0.0.0:{port}").parse().unwrap();
    grpc::start(addr, &options).await?;

    Ok(())
}

fn print_logo() {
    let banner = r#"
    _______      _____        
    ___    |_______  /________
    __  /| |  ___/  __/_  ___/
    _  ___ / /__ / /_ _(__  ) 
    /_/  |_\___/ \__/ /____/  
"#;
    println!("{}", banner);
    println!("created by Yao");
}
