use crate::{config::models::Config, server::dedicated::collection::DedicatedServers};

pub struct CtxManager {
    config: Config,
    connection: redis::Connection,
}

impl CtxManager {
    pub fn get_dedicated_servers(&self) -> &mut DedicatedServers {
        &mut self.config.dedicated_servers
    }

    pub fn get_config(&self) -> Config {
        self.config
    }

    pub fn get_connection(&self) -> redis::Connection {
        self.connection
    }

    pub fn new() -> Self {
        let config = Config::get_config();
        let connection = config.get_redis_connection();
        Self { config, connection }
    }

    pub fn from_config(config: &Config) -> Self {
        let connection = config.get_redis_connection();
        Self {
            config: config.clone(),
            connection,
        }
    }
}
