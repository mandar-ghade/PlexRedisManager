use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Local};
use redis::{FromRedisValue, RedisError};
use strum_macros::EnumString;
use thiserror::Error;

use crate::{context_manager::ContextManager, game::r#type::GameType, region::Region};

use super::server_group::ServerGroup;

#[derive(Error, Debug)]
pub enum MinecraftServerError {
    #[error("Parsing Error: `{0}`")]
    ParsingError(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ServerMotd {
    GameMotd(GameInfo),
    Motd(String),
}

#[allow(non_camel_case_types)]
#[derive(Clone, EnumString, Eq, Debug, PartialEq)]
enum GameDisplayStatus {
    ALWAYS_OPEN,
    STARTING,
    VOTING,
    WAITING,
    IN_PROGRESS,
    CLOSING,
}

#[allow(non_camel_case_types)]
#[derive(Clone, EnumString, Eq, Debug, PartialEq)]
enum GameJoinStatus {
    OPEN,
    RANKS_ONLY,
    CLOSED,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct GameInfo {
    game: GameType,
    mode: Option<String>,
    map: Option<String>,
    timer: i8,
    voting_on: Option<String>,
    host_rank: Option<String>,
    display_status: GameDisplayStatus,
    join_status: GameJoinStatus,
}

fn json_to_map(
    value: &serde_json::Value,
) -> Result<serde_json::Map<String, serde_json::Value>, MinecraftServerError> {
    match value {
        serde_json::Value::Object(map) => Ok(map.clone()),
        _ => Err(
            "Object not recognized (expected Map<String, serde_json::Value>)"
                .to_string()
                .into(),
        ),
    }
}

fn parse_game_type(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<GameType, MinecraftServerError> {
    match map.get(key) {
        Some(serde_json::Value::String(name)) => Ok(GameType::from_str(name.as_str()).map_err(
            |_| -> MinecraftServerError {
                format!("GameInfo Parsing Error: GameType: {:?} not found", name).into()
            },
        )?),
        Some(resp) => Err(format!(
            "GameInfo could not parse `{}` (expected string). Resp: {:?}",
            key, resp
        )
        .into()),
        None => Err(format!("GameInfo parsing error: Could not find key `{}`", key).into()),
    }
}

fn parse_optional_string_from_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    match map.get(key) {
        Some(serde_json::Value::String(value)) => Some(value.to_string()),
        Some(_) => None,
        None => None,
    }
}

fn parse_string_from_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<String, MinecraftServerError> {
    match map.get(key) {
        Some(serde_json::Value::String(value)) => Ok(value.to_string()),
        Some(resp) => Err(format!(
            "GameInfo could not parse `{}` (expected string). Resp: {:?}",
            key, resp
        )
        .into()),
        None => Err(format!("GameInfo parsing error: Could not find key `{}`", key).into()),
    }
}

fn parse_number_as_i64(key: &str, value: &serde_json::Value) -> Result<i64, MinecraftServerError> {
    value
        .as_i64()
        .ok_or(format!("Failed to convert `{}` to i64", key).into())
}

fn parse_u64_from_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<u64, MinecraftServerError> {
    match map.get(key) {
        Some(serde_json::Value::Number(value)) => {
            match parse_number_as_i64(key, &serde_json::Value::Number(value.clone())) {
                Ok(num) => Ok(num as u64),
                Err(MinecraftServerError::ParsingError(err_msg)) => {
                    Err(format!("Could not parse `{}` into u64: {}", key, err_msg).into())
                }
            }
        }
        Some(_) => Err(format!("Could not parse `{}` (expected Number)", key).into()),
        None => Err(format!("Parsing error: Could not find key `{}`", key).into()),
    }
}

fn parse_u16_from_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<u16, MinecraftServerError> {
    match map.get(key) {
        Some(serde_json::Value::Number(value)) => {
            match parse_number_as_i64(key, &serde_json::Value::Number(value.clone())) {
                Ok(num) => Ok(num as u16),
                Err(MinecraftServerError::ParsingError(err_msg)) => {
                    Err(format!("Could not parse `{}` into u16: {}", key, err_msg).into())
                }
            }
        }
        Some(_) => Err(format!("Could not parse `{}` (expected Number)", key).into()),
        None => Err(format!("Parsing error: Could not find key `{}`", key).into()),
    }
}

fn parse_u8_from_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<u8, MinecraftServerError> {
    match map.get(key) {
        Some(serde_json::Value::Number(value)) => {
            match parse_number_as_i64(key, &serde_json::Value::Number(value.clone())) {
                Ok(num) => Ok(num as u8),
                Err(MinecraftServerError::ParsingError(err_msg)) => {
                    Err(format!("Could not parse `{}` into u8: {}", key, err_msg).into())
                }
            }
        }
        Some(_) => Err(format!("Could not parse `{}` (expected Number)", key).into()),
        None => Err(format!("Parsing error: Could not find key `{}`", key).into()),
    }
}
fn parse_i8_from_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<i8, MinecraftServerError> {
    match map.get(key) {
        Some(serde_json::Value::Number(value)) => {
            match parse_number_as_i64(key, &serde_json::Value::Number(value.clone())) {
                Ok(num) => Ok(num as i8),
                Err(MinecraftServerError::ParsingError(err_msg)) => {
                    Err(format!("GameInfo could not parse `{}` into i8: {}", key, err_msg).into())
                }
            }
        }
        Some(_) => Err(format!("GameInfo could not parse `{}` (expected Number)", key).into()),
        None => Err(format!("GameInfo parsing error: Could not find key `{}`", key).into()),
    }
}

fn parse_display_status_from_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<GameDisplayStatus, MinecraftServerError> {
    Ok(
        GameDisplayStatus::from_str(&parse_string_from_map(map, key)?).map_err(|_| {
            format!(
                "GameInfo could not parse `{}` (Expected GameDisplayStatus)",
                key
            )
        })?,
    )
}

fn parse_join_status_from_map(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<GameJoinStatus, MinecraftServerError> {
    Ok(
        GameJoinStatus::from_str(&parse_string_from_map(map, key)?).map_err(|_| {
            format!(
                "GameInfo could not parse `{}` (Expected GameJoinStatus)",
                key
            )
        })?,
    )
}

impl GameInfo {
    fn parse_motd(
        map: serde_json::Map<String, serde_json::Value>,
    ) -> Result<ServerMotd, MinecraftServerError> {
        Ok(ServerMotd::GameMotd(Self {
            game: parse_game_type(&map, "_game")?,
            mode: parse_optional_string_from_map(&map, "_mode"),
            map: parse_optional_string_from_map(&map, "_map"),
            timer: parse_i8_from_map(&map, "_timer")?,
            voting_on: parse_optional_string_from_map(&map, "_votingOn"),
            host_rank: parse_optional_string_from_map(&map, "_hostRank"),
            display_status: parse_display_status_from_map(&map, "_status")?,
            join_status: parse_join_status_from_map(&map, "_joinable")?,
        }))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MinecraftServer {
    name: String,
    group: String,
    motd: ServerMotd,
    player_count: u8,
    max_player_count: u8,
    tps: u16,
    ram: u16,
    max_ram: u16,
    public_address: String,
    port: u16,
    donors_online: u8,
    start_up_date: u64, // seconds since epoch
    current_time: u64,  // ms since epoch
}

impl From<String> for MinecraftServerError {
    fn from(value: String) -> Self {
        Self::ParsingError(value.into())
    }
}

fn parse_datetime(
    map: &HashMap<String, String>,
    key: &str,
) -> Result<DateTime<Local>, MinecraftServerError> {
    todo!()
}

fn parse_json_motd(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<ServerMotd, MinecraftServerError> {
    match map.get(key) {
        Some(serde_json::Value::String(string)) => Ok(ServerMotd::Motd(string.to_string())),
        Some(serde_json::Value::Object(map)) => Ok(GameInfo::parse_motd(map.clone())?),
        Some(resp) => Err(format!(
            "GameInfo could not parse `{}` into MotdJson (expected object or string). Resp: {:?}",
            key, resp
        )
        .into()),
        None => Err(format!(
            "GameInfo parsing error: Could not find key `{}` while parsing MotdJson",
            key
        )
        .into()),
    }
}

impl TryFrom<serde_json::Value> for MinecraftServer {
    type Error = MinecraftServerError;
    fn try_from(map: serde_json::Value) -> Result<Self, Self::Error> {
        let map: serde_json::Map<String, serde_json::Value> = json_to_map(&map)?;
        Ok(Self {
            name: parse_string_from_map(&map, "_name")?,
            group: parse_string_from_map(&map, "_group")?,
            motd: parse_json_motd(&map, "_motd")?,
            player_count: parse_u8_from_map(&map, "_playerCount")?,
            max_player_count: parse_u8_from_map(&map, "_maxPlayerCount")?,
            tps: parse_u16_from_map(&map, "_tps")?,
            ram: parse_u16_from_map(&map, "_ram")?,
            max_ram: parse_u16_from_map(&map, "_maxRam")?,
            public_address: parse_string_from_map(&map, "_publicAddress")?,
            port: parse_u16_from_map(&map, "_port")?,
            donors_online: parse_u8_from_map(&map, "_donorsOnline")?,
            start_up_date: parse_u64_from_map(&map, "_startUpDate")?,
            current_time: parse_u64_from_map(&map, "_currentTime")?,
        })
    }
}

impl From<MinecraftServerError> for RedisError {
    fn from(err: MinecraftServerError) -> Self {
        match err {
            MinecraftServerError::ParsingError(err_msg) => (
                redis::ErrorKind::ParseError,
                "Server cache parsing error",
                err_msg,
            )
                .into(),
        }
    }
}

impl FromRedisValue for MinecraftServer {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let map_str: String = redis::from_redis_value(v)?;
        let map: serde_json::Value = serde_json::from_str(map_str.as_str()).map_err(|_| {
            RedisError::from(MinecraftServerError::from(
                "Error parsing Minecraft Server cache".to_string(),
            ))
        })?;
        let res: Result<MinecraftServer, MinecraftServerError> = map.try_into();
        let mapped_res: Result<MinecraftServer, RedisError> = res.map_err(|err| err.into());
        mapped_res
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum ServerStatus {
    ONLINE,
    OFFLINE,
    DOES_NOT_EXIST,
    GROUP_NOT_FOUND,
    INSTANCE_NOT_FOUND,
}

impl MinecraftServer {
    fn get_server_group(&self, ctx: &mut ContextManager) -> Option<ServerGroup> {
        let key: String = format!("servergroups.{}", self.group);
        ServerGroup::from_str(key.as_str(), ctx).ok()
    }

    pub fn from_server_group(
        server_group: &ServerGroup,
        ctx: &mut ContextManager,
    ) -> Result<Vec<Self>, MinecraftServerError> {
        let server_statuses: Vec<String> = redis::cmd("KEYS")
            .arg(format!(
                "serverstatus.minecraft.{}.{}-*",
                server_group.region, server_group.prefix
            ))
            .query(ctx.get_connection())
            .map_err(|_| -> MinecraftServerError {
                "Redis data for MinecraftServer could not be retrieved. MinecraftServer iteration failed."
                    .to_string().into()
            })?;
        server_statuses
            .iter()
            .map(|sg| Self::get_from_raw_str(sg.as_str(), ctx))
            .collect()
    }

    fn is_online(&self) -> bool {
        //! Checks whether the current timestamp in milliseconds
        //! falls between 5 seconds of `self.current_time`
        let now = Local::now().timestamp_millis() as u64;
        let seconds = 5; // 5 * 2 = 10 seconds (happened just 5-10 seconds ago)
        let interval = seconds * 1000; // in milliseconds
        let seconds_before_curr = self.current_time - interval;
        let seconds_after_curr = self.current_time + interval;
        seconds_before_curr <= now && now <= seconds_after_curr
    }

    /// Gets current ServerStatus
    /// Updates `self` if it is online.
    /// If offline, please do not use it (delete it from vec or whatever).
    pub fn update(&mut self, ctx: &mut ContextManager) -> ServerStatus {
        let Some(group) = self.get_server_group(ctx) else {
            return ServerStatus::GROUP_NOT_FOUND;
        };
        let Ok(server) = Self::get(&self.name, &group.region, ctx) else {
            return ServerStatus::INSTANCE_NOT_FOUND;
        };
        if self.current_time == server.current_time && !self.is_online() {
            return ServerStatus::OFFLINE;
        }
        *self = server;
        ServerStatus::ONLINE
    }

    fn get_uptime_as_seconds(&self) -> i64 {
        return Local::now().timestamp() - (self.start_up_date as i64);
    }

    fn get_prefix(&self) -> u8 {
        let (_, prefix) = self
            .name
            .split_once('-')
            .unwrap_or((self.name.as_str(), "0"));
        prefix.parse().unwrap_or(0)
    }

    fn is_empty(&self) -> bool {
        return self.player_count == 0;
    }

    fn is_dead_server(&self) -> bool {
        //? Returns `true` if player_count is None and server has been online for over 2 minutes.
        return self.is_empty() && self.get_uptime_as_seconds() >= 150;
    }

    pub fn get_empty_servers(ctx: &mut ContextManager) -> Result<Vec<Self>, MinecraftServerError> {
        Ok(Self::get_all(ctx)?
            .into_iter()
            .filter(|sv| sv.is_dead_server()) // offline
            .collect())
    }

    pub fn get_all(ctx: &mut ContextManager) -> Result<Vec<Self>, MinecraftServerError> {
        let server_statuses: Vec<String> = redis::cmd("KEYS")
            .arg("serverstatus.minecraft.*.*")
            .query(ctx.get_connection())
            .map_err(|_| -> MinecraftServerError {
                "Redis data for MinecraftServer could not be retrieved. MinecraftServer iteration failed."
                    .to_string().into()
            })?;
        server_statuses
            .iter()
            .map(|ss| Self::get_from_raw_str(ss.as_str(), ctx))
            .collect()
    }

    fn get_from_raw_str(key: &str, ctx: &mut ContextManager) -> Result<Self, MinecraftServerError> {
        redis::cmd("GET")
            .arg(key)
            .query(ctx.get_connection())
            .map_err(|err| {
                format!("Redis data for {:?} could not be retrieved: {:?}", key, err)
                    .to_string()
                    .into()
            })
    }

    pub fn get(
        server_name: &String,
        region: &Region,
        ctx: &mut ContextManager,
    ) -> Result<Self, MinecraftServerError> {
        let key: String = format!(
            "serverstatus.minecraft.{}.{}",
            region.to_string(),
            server_name
        );
        Self::get_from_raw_str(key.as_str(), ctx)
    }
}
