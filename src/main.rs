use lazy_static::lazy_static;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    fs::{self, File},
    io::Read,
    str::FromStr,
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Debug)]
struct Game {
    name: GameType,
    options: GameOptions,
}

impl Game {
    fn from_str(game: &str) -> Option<Self> {
        if let Some(game_name) = GameType::from_str(game).ok() {
            Self {
                name: game_name,
                options: GameOptions::from(&game_name),
            };
        }
        None
    }
    fn from(game: &GameType) -> Self {
        Self {
            name: *game,
            options: GameOptions::from(&game),
        }
    }
}

#[derive(Clone, Debug)]
struct GameOptions {
    prefix: String,
    team_server: Option<GameType>,
    booster_group: Option<BoosterGroup>,
    npc_name: Option<String>,
    min_players: u8,
    max_players: u8,
}

impl GameOptions {
    fn from(game: &GameType) -> Self {
        let (min_players, max_players) = GAME_TO_PLAYER_COUNT.get(game).cloned().unwrap_or((8, 16));
        Self {
            prefix: GAME_TO_SERVER_PREFIX
                .get(game)
                .cloned()
                .unwrap_or_default()
                .into(),
            team_server: GAME_TO_TEAM_SERVER.get(game).cloned(),
            booster_group: GAME_TO_BOOSTER_GROUP.get(game).cloned(),
            npc_name: GAME_TO_NPC.get(game).cloned().map(|x| x.to_string()),
            min_players,
            max_players,
        }
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Hash, EnumString, EnumIter)]
enum GameType {
    Micro,
    MixedArcade,
    Draw,
    Build,
    TurfWars,
    SpeedBuilders,
    HideSeek,
    CakeWarsDuos,
    CakeWars4,
    SurvivalGames,
    SurvivalGamesTeams,
    Skywars,
    SkywarsTeams,
    Bridges,
    MineStrike,
    Smash,
    SmashTeams,
    ChampionsDOM,
    ChampionsCTF,
    Clans,
    ClansHub,
    BaconBrawl,
    Lobbers,
    DeathTag,
    DragonEscape,
    Dragons,
    Evolution,
    MilkCow,
    Paintball,
    Quiver,
    Runner,
    Sheep,
    Snake,
    SneakyAssassins,
    Spleef,
    SquidShooters,
    WitherAssault,
}

lazy_static! {
    static ref GAME_TO_NPC: HashMap<GameType, &'static str> = HashMap::from([
        (GameType::Build, "Master Builders"),
        (GameType::Draw, "Draw My Thing"),
        (GameType::Micro, "Micro Battles"),
        (GameType::MixedArcade, "Mixed Arcade"),
        (GameType::TurfWars, "Turf Wars"),
        (GameType::SpeedBuilders, "Speed Builders"),
        (GameType::HideSeek, "Block Hunt"),
        (GameType::CakeWars4, "Cake Wars"),
        (GameType::SurvivalGames, "Survival Games"),
        (GameType::Skywars, "Skywars"),
        (GameType::Bridges, "The Bridges"),
        (GameType::MineStrike, "Mine-Strike"),
        (GameType::Smash, "Super Smash Mobs"),
        (GameType::Clans, "Clans"),
        (GameType::ClansHub, "ClansHub"),
    ]);
    static ref GAME_TO_BOOSTER_GROUP: HashMap<GameType, BoosterGroup> = HashMap::from([
        (GameType::Micro, BoosterGroup::Arcade),
        (GameType::Draw, BoosterGroup::Draw_My_Thing),
        (GameType::MineStrike, BoosterGroup::Arcade),
        (GameType::TurfWars, BoosterGroup::Arcade),
        (GameType::Build, BoosterGroup::Master_Builders),
        (GameType::SpeedBuilders, BoosterGroup::Speed_Builders),
        (GameType::HideSeek, BoosterGroup::Block_Hunt),
        (GameType::CakeWarsDuos, BoosterGroup::Cake_Wars),
        (GameType::CakeWars4, BoosterGroup::Cake_Wars),
        (GameType::SurvivalGames, BoosterGroup::Survival_Games),
        (GameType::SurvivalGamesTeams, BoosterGroup::Survival_Games),
        (GameType::Skywars, BoosterGroup::Skywars),
        (GameType::SkywarsTeams, BoosterGroup::Skywars),
        (GameType::Bridges, BoosterGroup::Bridges),
        (GameType::MineStrike, BoosterGroup::MineStrike),
        (GameType::Smash, BoosterGroup::Smash_Mobs),
        (GameType::SmashTeams, BoosterGroup::Smash_Mobs),
        (GameType::ChampionsDOM, BoosterGroup::Champions),
        (GameType::ChampionsCTF, BoosterGroup::Champions)
    ]);
    static ref GAME_TO_PLAYER_COUNT: HashMap<GameType, (u8, u8)> = HashMap::from([
        (GameType::Micro, (8, 16)),
        (GameType::MixedArcade, (8, 24)),
        (GameType::Draw, (5, 8)),
        (GameType::Build, (8, 12)),
        (GameType::TurfWars, (8, 16)),
        (GameType::SpeedBuilders, (4, 8)),
        (GameType::HideSeek, (12, 24)),
        (GameType::CakeWarsDuos, (10, 16)),
        (GameType::CakeWars4, (10, 16)),
        (GameType::SurvivalGames, (12, 24)),
        (GameType::SurvivalGamesTeams, (12, 24)),
        (GameType::Skywars, (8, 12)),
        (GameType::SkywarsTeams, (8, 12)),
        (GameType::Bridges, (20, 40)),
        (GameType::MineStrike, (8, 16)),
        (GameType::Smash, (4, 6)),
        (GameType::SmashTeams, (4, 6)),
        (GameType::ChampionsDOM, (8, 10)),
        (GameType::ChampionsCTF, (10, 16)),
        (GameType::Clans, (1, 50)),
        (GameType::ClansHub, (1, 50)),
    ]);
    static ref GAME_TO_SERVER_PREFIX: HashMap<GameType, &'static str> = HashMap::from([
        (GameType::Micro, "MB"),
        (GameType::MixedArcade, "MIN"),
        (GameType::Draw, "DMT"),
        (GameType::Build, "BLD"),
        (GameType::TurfWars, "TF"),
        (GameType::SpeedBuilders, "SB"),
        (GameType::HideSeek, "BH"),
        (GameType::CakeWarsDuos, "CW2"),
        (GameType::CakeWars4, "CW4"),
        (GameType::SurvivalGames, "HG"),
        (GameType::SurvivalGamesTeams, "SG2"),
        (GameType::Skywars, "SKY"),
        (GameType::SkywarsTeams, "SKY2"),
        (GameType::Bridges, "BR"),
        (GameType::MineStrike, "MS"),
        (GameType::Smash, "SSM"),
        (GameType::SmashTeams, "SSM2"),
        (GameType::ChampionsDOM, "DOM"),
        (GameType::ChampionsCTF, "CTF"),
        (GameType::Clans, "Clans"),
        (GameType::ClansHub, "ClansHub"),
    ]);
    static ref GAME_TO_TEAM_SERVER: HashMap<GameType, GameType> = HashMap::from([
        (GameType::Skywars, GameType::SkywarsTeams),
        (GameType::SurvivalGames, GameType::SurvivalGamesTeams),
        (GameType::Smash, GameType::SmashTeams),
        (GameType::ChampionsDOM, GameType::ChampionsCTF),
        (GameType::CakeWars4, GameType::CakeWarsDuos),
    ]);
}

#[derive(Clone, Copy, Debug, Display)]
#[allow(non_camel_case_types)]
enum BoosterGroup {
    Arcade,
    Draw_My_Thing,
    Master_Builders,
    Speed_Builders,
    Block_Hunt,
    Cake_Wars,
    Survival_Games,
    Skywars,
    Bridges,
    MineStrike,
    Smash_Mobs,
    Champions,
}

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
    #[serde(deserialize_with = "to_u16")]
    ram: u16,
    #[serde(deserialize_with = "to_u8")]
    cpu: u8,
    #[serde(deserialize_with = "to_u8")]
    total_servers: u8,
    #[serde(deserialize_with = "to_u8")]
    joinable_servers: u8,
    #[serde(deserialize_with = "to_u16")]
    port_section: u16,
    uptimes: Option<String>,
    #[serde(deserialize_with = "to_bool")]
    arcade_group: bool,
    world_zip: String,
    plugin: String,
    config_path: String,
    host: Option<String>,
    #[serde(deserialize_with = "to_u8")]
    min_players: u8,
    #[serde(deserialize_with = "to_u8")]
    max_players: u8,
    #[serde(deserialize_with = "to_bool")]
    pvp: bool,
    #[serde(deserialize_with = "to_bool")]
    tournament: bool,
    #[serde(deserialize_with = "to_bool")]
    tournament_points: bool,
    #[serde(default, deserialize_with = "to_bool")]
    hard_max_player_cap: bool,
    games: String,
    modes: Option<String>,
    booster_group: Option<String>,
    server_type: String,
    #[serde(deserialize_with = "to_bool")]
    add_no_cheat: bool,
    #[serde(deserialize_with = "to_bool")]
    add_world_edit: bool,
    #[serde(deserialize_with = "to_bool")]
    team_rejoin: bool,
    #[serde(deserialize_with = "to_bool")]
    team_auto_join: bool,
    #[serde(deserialize_with = "to_bool")]
    team_force_balance: bool,
    #[serde(deserialize_with = "to_bool")]
    game_auto_start: bool,
    #[serde(deserialize_with = "to_bool")]
    game_timeout: bool,
    #[serde(default, deserialize_with = "to_bool")]
    game_voting: bool,
    #[serde(default, deserialize_with = "to_bool")]
    map_voting: bool,
    #[serde(deserialize_with = "to_bool")]
    reward_gems: bool,
    #[serde(deserialize_with = "to_bool")]
    reward_items: bool,
    #[serde(deserialize_with = "to_bool")]
    reward_stats: bool,
    #[serde(deserialize_with = "to_bool")]
    reward_achievements: bool,
    #[serde(deserialize_with = "to_bool")]
    hotbar_inventory: bool,
    #[serde(deserialize_with = "to_bool")]
    hotbar_hub_clock: bool,
    #[serde(deserialize_with = "to_bool")]
    player_kick_idle: bool,
    #[serde(default, deserialize_with = "to_bool")]
    staff_only: bool,
    #[serde(default, deserialize_with = "to_bool")]
    whitelist: bool,
    resource_pack: Option<String>,
    #[serde(default, deserialize_with = "to_region")]
    region: Region,
    team_server_key: Option<String>,
    portal_bottom_corner_location: Option<String>,
    portal_top_corner_location: Option<String>,
    npc_name: Option<String>,
}

fn to_region<'de, D>(deserializer: D) -> Result<Region, D::Error>
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

fn to_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    s.parse::<u16>().map_err(de::Error::custom)
}

fn to_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    s.parse::<u8>().map_err(de::Error::custom)
}

fn to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
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
        .arg("servergroups.Lobby")
        .query(&mut conn)
        .expect("Redis data for ServerGroup should have been found");
    if output.is_empty() {
        return println!("Redis data not found");
    }
    let sg_str = serde_json::to_string(&output).unwrap();
    let sg: ServerGroup = serde_json::from_str(&sg_str).unwrap();
    dbg!(&sg);
    dbg!(Game::from(&GameType::MixedArcade));
    //GameType::iter().for_each(|g| println!("{:?}", Game::from(&g)));
}
