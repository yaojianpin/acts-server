use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub data_dir: Option<String>,
    pub log: Option<ConfigLog>,
    pub port: Option<u32>,
}

#[derive(Deserialize)]
pub struct ConfigLog {
    pub dir: Option<String>,
    pub level: Option<String>,
}
