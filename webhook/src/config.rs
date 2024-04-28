use std::{collections::BTreeMap, env::var, fs::File, io::Read, sync::OnceLock};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Service {
    pub pre_start_command: Option<String>,
    pub start_command: Option<String>,
    pub post_start_command: Option<String>,
    pub stop_command: Option<String>,
}

#[derive(Deserialize)]
pub struct Config {
    pub secret: String,
    pub default: Service,
    pub services: BTreeMap<String, Service>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let path = var("CONFIG_PATH").unwrap_or("/etc/webhook/config.json".to_owned());
        log::info!("Loading config from {path}");

        let mut file = File::open(path).expect("Unable to open config file");
        let mut str: String = String::new();
        file.read_to_string(&mut str)
            .expect("Unable to read from config file");
        serde_json::from_str::<Config>(str.as_str()).expect("Unable to parse config file")
    })
}
