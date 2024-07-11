use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Read,
};

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
enum Region {
    US,
    EU,
    ALL,
}

impl Default for Region {
    fn default() -> Self {
        Region::US
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerGroup {
    name: String,
    prefix: String,
    #[serde(deserialize_with = "from_u16")]
    ram: u16,
    #[serde(deserialize_with = "from_u8")]
    cpu: u8,
    #[serde(deserialize_with = "from_u8")]
    total_servers: u8,
    #[serde(deserialize_with = "from_u8")]
    joinable_servers: u8,
    #[serde(deserialize_with = "from_u16")]
    port_section: u16,
    uptimes: Option<String>,
    #[serde(deserialize_with = "from_bool")]
    arcade_group: bool,
    world_zip: String,
    plugin: String,
    config_path: String,
    host: Option<String>,
    #[serde(deserialize_with = "from_u8")]
    min_players: u8,
    #[serde(deserialize_with = "from_u8")]
    max_players: u8,
    #[serde(deserialize_with = "from_bool")]
    pvp: bool,
    #[serde(deserialize_with = "from_bool")]
    tournament: bool,
    #[serde(deserialize_with = "from_bool")]
    tournament_points: bool,
    #[serde(default, deserialize_with = "from_bool")]
    hard_max_player_cap: bool,
    games: String,
    modes: Option<String>,
    booster_group: Option<String>,
    server_type: String,
    #[serde(deserialize_with = "from_bool")]
    add_no_cheat: bool,
    #[serde(deserialize_with = "from_bool")]
    add_world_edit: bool,
    #[serde(deserialize_with = "from_bool")]
    team_rejoin: bool,
    #[serde(deserialize_with = "from_bool")]
    team_auto_join: bool,
    #[serde(deserialize_with = "from_bool")]
    team_force_balance: bool,
    #[serde(deserialize_with = "from_bool")]
    game_auto_start: bool,
    #[serde(deserialize_with = "from_bool")]
    game_timeout: bool,
    #[serde(default, deserialize_with = "from_bool")]
    game_voting: bool,
    #[serde(default, deserialize_with = "from_bool")]
    map_voting: bool,
    #[serde(deserialize_with = "from_bool")]
    reward_gems: bool,
    #[serde(deserialize_with = "from_bool")]
    reward_items: bool,
    #[serde(deserialize_with = "from_bool")]
    reward_stats: bool,
    #[serde(deserialize_with = "from_bool")]
    reward_achievements: bool,
    #[serde(deserialize_with = "from_bool")]
    hotbar_inventory: bool,
    #[serde(deserialize_with = "from_bool")]
    hotbar_hub_clock: bool,
    #[serde(deserialize_with = "from_bool")]
    player_kick_idle: bool,
    #[serde(default, deserialize_with = "from_bool")]
    staff_only: bool,
    #[serde(default, deserialize_with = "from_bool")]
    whitelist: bool,
    resource_pack: Option<String>,
    #[serde(default, deserialize_with = "from_region")]
    region: Region,
    team_server_key: Option<String>,
    portal_bottom_corner_location: Option<String>,
    portal_top_corner_location: Option<String>,
    npc_name: Option<String>,
}

fn from_region<'de, D>(deserializer: D) -> Result<Region, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    match s {
        "ALL" => Ok(Region::ALL),
        "US" => Ok(Region::US),
        "EU" => Ok(Region::EU),
        _ => Err(de::Error::unknown_variant(s, &["ALL", "US", "EU"])),
    }
}

fn from_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    s.parse::<u16>().map_err(de::Error::custom)
}

fn from_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    s.parse::<u8>().map_err(de::Error::custom)
}

fn from_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(de::Error::unknown_variant(s, &["true", "false"])),
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    redis_conn: RedisConfig,
}

#[derive(Deserialize, Serialize, Debug)]
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

fn connect(config: &Config) -> redis::Connection {
    redis::Client::open(format!(
        "redis://{}:{}",
        config.redis_conn.address, config.redis_conn.port
    ))
    .expect("Redis connection could not be made.")
    .get_connection()
    .expect("Redis client could not be opened.")
}

fn main() {
    let config: Config = Config::get_config();
    dbg!(&config);
    let mut conn = connect(&config);
    let output: BTreeMap<String, String> = redis::cmd("HGETALL")
        .arg("servergroups.Clans")
        .query(&mut conn)
        .expect("Redis data for ServerGroup should have been found");
    let sg_str = serde_json::to_string(&output).unwrap();
    dbg!(&sg_str);
    let sg: ServerGroup = serde_json::from_str(&sg_str).unwrap();
    dbg!(&sg);
    dbg!(&sg.region);
}
