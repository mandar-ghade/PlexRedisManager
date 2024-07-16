use crate::config::models::Config;

pub fn connect(config: &Config) -> redis::Connection {
    redis::Client::open(format!(
        "redis://{}:{}",
        config.redis_conn.address, config.redis_conn.port
    ))
    .expect("Redis connection could not be made")
    .get_connection()
    .expect("Redis client could not be opened")
}
