use std::fs;

use dirs::home_dir;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_token: String,
    pub access_secret: String,
}

impl Config {
    pub fn load() -> Self {
        let config_dir = home_dir()
            .expect("Home dir not found!")
            .join(".config/twitter_cli/config.toml");

        let data = fs::read_to_string(config_dir).expect("Could not read config.");

        toml::from_str(&data).expect("Invalid config format.")
    }
}
