use std::{
    fs::{self, File},
    io::Read,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    redis_conn: RedisConfig,
}

#[derive(Deserialize, Serialize)]
struct RedisConfig {
    address: String,
    port: String,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            address: String::from("127.0.0.1"),
            port: String::from("6379"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            redis_conn: RedisConfig::default(),
        }
    }
}

impl Config {
    fn get_config() -> Self {
        let mut file = File::open("config.toml").expect("File should have been expected.");
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)
            .expect("Cannot read contents of file.");
        let cfg: Self = match toml::from_str(&toml_str) {
            Ok(cfg) => cfg,
            Err(_) => {
                let default = Self::default();
                let contents = toml::to_string(&default)
                    .expect("Default toml config could not be formatted to string.");
                let _ = fs::write("config.toml", contents);
                return default;
            }
        };
        cfg
    }
}

fn main() {
    let cfg: Config = Config::get_config();
    println!(
        "Address: {:?}, Port: {:?}",
        cfg.redis_conn.address, cfg.redis_conn.port
    )
}
