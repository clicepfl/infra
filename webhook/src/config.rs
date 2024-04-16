use std::{collections::BTreeMap, env::var, fs::File, io::Read, sync::OnceLock};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Service {
    pub start_command: String,
    pub stop_command: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub secret: String,
    #[serde(rename = "ref")]
    pub ref_: String,
    pub generic_start_command: String,
    pub services: BTreeMap<String, Service>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let mut file =
            File::open(var("CONFIG_PATH").unwrap_or("/etc/webhook/config.json".to_owned()))
                .expect("Unable to open config file");
        let mut str: String = String::new();
        file.read_to_string(&mut str)
            .expect("Unable to read from config file");
        serde_json::from_str::<Config>(str.as_str()).expect("Unable to parse config file")
    })
}
