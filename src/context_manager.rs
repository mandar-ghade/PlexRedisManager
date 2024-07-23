use crate::{config::models::Config, server::dedicated::collection::DedicatedServers};

pub struct ContextManager {
    config: Config,
    connection: redis::Connection,
}

impl ContextManager {
    pub fn get_dedicated_servers(&mut self) -> &mut DedicatedServers {
        &mut self.config.dedicated_servers
    }

    pub fn get_config(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn get_connection(&mut self) -> &mut redis::Connection {
        &mut self.connection
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
