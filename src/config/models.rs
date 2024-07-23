use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{
    region::Region,
    server::dedicated::{
        collection::DedicatedServers, server::DedicatedServer, System, SystemName,
    },
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    redis_conn: RedisConfig,
    pub sys_info: System,
    pub monitor_info: MonitorInfo,
    pub dedicated_servers: DedicatedServers,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RedisConfig {
    pub address: String,
    pub port: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MonitorInfo {
    scripts_path: String, // should be turned in to Path objects
    worlds_path: String,
    config_path: String,
}

impl Default for MonitorInfo {
    fn default() -> Self {
        Self {
            scripts_path: "/home/mineplex".into(),
            worlds_path: "/home/mineplex/worlds".into(),
            config_path: "/home/mineplex/configs".into(),
        }
    }
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
            sys_info: System {
                system: SystemName::Linux,
            },
            monitor_info: MonitorInfo::default(),
            dedicated_servers: DedicatedServers {
                servers: Vec::new(),
            },
        }
    }
}

fn dedicated_server_with_defaults(ds: &mut DedicatedServer) -> DedicatedServer {
    ds.max_ram = ds.available_ram;
    ds.max_cpu = ds.available_cpu;
    ds.server_count_map = HashMap::new();
    ds.clone()
}

impl Config {
    pub fn get_redis_connection(&self) -> redis::Connection {
        redis::Client::open(format!(
            "redis://{}:{}",
            self.redis_conn.address, self.redis_conn.port
        ))
        .expect("Redis connection could not be made")
        .get_connection()
        .expect("Redis client could not be opened")
    }

    pub fn get_config() -> Self {
        let mut file = File::open("config.toml").expect("File should have been expected.");
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)
            .expect("Cannot read contents of file.");
        let mut cfg: Self = match toml::from_str(&toml_str) {
            Ok(cfg) => cfg,
            Err(_) => {
                let default = Self::default();
                let contents = toml::to_string(&default)
                    .expect("Default toml config could not be formatted to string.");
                let _ = fs::write("config.toml", contents);
                return default;
            }
        };
        let modified_servers: Vec<DedicatedServer> = cfg
            .dedicated_servers
            .servers
            .clone()
            .into_iter()
            .map(|mut sv| dedicated_server_with_defaults(&mut sv))
            .collect();
        cfg.dedicated_servers.servers = modified_servers;
        cfg
    }
}
