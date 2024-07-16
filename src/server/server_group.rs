use crate::config::models::Config;
use crate::error::parsing_error::ServerGroupParsingError;
use crate::game::utils::GAME_TO_SERVER_PREFIX;
use crate::game::Game;
use crate::redis::connect;
use crate::region::Region;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ServerGroup {
    pub name: String,
    pub prefix: String,
    pub ram: u16,
    pub cpu: u8,
    pub total_servers: u8,
    pub joinable_servers: u8,
    pub port_section: u16,
    pub uptimes: Option<String>,
    pub arcade_group: bool,
    pub world_zip: String,
    pub plugin: String,
    pub config_path: String,
    pub host: Option<String>,
    pub min_players: u8,
    pub max_players: u8,
    pub pvp: bool,
    pub tournament: bool,
    pub tournament_points: bool,
    pub hard_max_player_cap: bool,
    pub games: Option<String>,
    pub modes: Option<String>,
    pub booster_group: Option<String>,
    pub server_type: String,
    pub add_no_cheat: bool,
    pub add_world_edit: bool,
    pub team_rejoin: bool,
    pub team_auto_join: bool,
    pub team_force_balance: bool,
    pub game_auto_start: bool,
    pub game_timeout: bool,
    pub game_voting: bool,
    pub map_voting: bool,
    pub reward_gems: bool,
    pub reward_items: bool,
    pub reward_stats: bool,
    pub reward_achievements: bool,
    pub hotbar_inventory: bool,
    pub hotbar_hub_clock: bool,
    pub player_kick_idle: bool,
    pub staff_only: bool,
    pub whitelist: bool,
    pub resource_pack: Option<String>,
    pub region: Region,
    pub team_server_key: Option<String>,
    pub portal_bottom_corner_location: Option<String>,
    pub portal_top_corner_location: Option<String>,
    pub npc_name: Option<String>,
}

fn parse_value<'a>(
    prefix: &String,
    map: &HashMap<String, String>,
    key: &'a str,
) -> Result<String, ServerGroupParsingError> {
    Ok(map
        .get(key)
        .ok_or(ServerGroupParsingError::new(format!(
            "servergroups.{} {:?} could not be found.",
            prefix, key
        )))?
        .to_string())
}

fn parse_bool_or_default<'a>(
    prefix: &String,
    map: &HashMap<String, String>,
    key: &'a str,
) -> Result<bool, ServerGroupParsingError> {
    match map.get(key).unwrap_or(&String::new()).as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        "null" | "" => Ok(false),
        _ => Err(ServerGroupParsingError::new(format!(
            "servergroups.{}: {:?} could not be found",
            prefix, key
        ))),
    }
}

fn parse_u8<'a>(
    prefix: &String,
    map: &HashMap<String, String>,
    key: &'a str,
) -> Result<u8, ServerGroupParsingError> {
    map.get(key)
        .ok_or(ServerGroupParsingError::new(format!(
            "servergroups.{}  {:?} (u8) could not be found.",
            prefix, key
        )))?
        .parse()
        .map_err(|err| {
            ServerGroupParsingError::new(format!(
                "servergroups.{}  {:?} (u8): {:?}",
                prefix, key, err
            ))
        })
}

fn parse_u16<'a>(
    prefix: &String,
    map: &HashMap<String, String>,
    key: &'a str,
) -> Result<u16, ServerGroupParsingError> {
    map.get(key)
        .ok_or(ServerGroupParsingError::new(format!(
            "servergroups.{}  {:?} (u16) could not be found",
            prefix, key
        )))?
        .parse()
        .map_err(|err| {
            ServerGroupParsingError::new(format!(
                "servergroups.{}  {:?} (u16): {:?}",
                prefix, key, err
            ))
        })
}

fn parse_optional_str<'a>(
    map: &HashMap<String, String>,
    key: &'a str,
) -> Result<Option<String>, ServerGroupParsingError> {
    Ok(map
        .get(key)
        .filter(|x| !x.is_empty() && x.as_str() != "null")
        .cloned())
}

impl From<Game> for ServerGroup {
    fn from(game: Game) -> Self {
        Self {
            name: game.options.prefix.clone(),
            prefix: game.options.prefix,
            ram: 512,
            cpu: 1,
            total_servers: 0,
            joinable_servers: 0,
            port_section: game.options.port_section,
            uptimes: None,
            arcade_group: game.options.arcade_group,
            world_zip: game.options.world_zip,
            plugin: game.options.plugin,
            config_path: game.options.config_path,
            host: game.options.host,
            min_players: game.options.min_players,
            max_players: game.options.max_players,
            pvp: game.options.pvp,
            tournament: game.options.tournament,
            tournament_points: game.options.tournament_points,
            hard_max_player_cap: false,
            games: game.options.games,
            modes: None,
            booster_group: game.options.booster_group.map(|b| b.to_string()),
            server_type: game.options.server_type,
            add_no_cheat: game.options.add_no_cheat,
            add_world_edit: game.options.add_world_edit,
            team_rejoin: game.options.team_rejoin,
            team_auto_join: game.options.team_auto_join,
            team_force_balance: game.options.team_force_balance,
            game_auto_start: game.options.game_auto_start,
            game_timeout: game.options.game_timeout,
            game_voting: game.options.game_voting,
            map_voting: game.options.map_voting,
            reward_gems: game.options.reward_gems,
            reward_items: game.options.reward_items,
            reward_stats: game.options.reward_stats,
            reward_achievements: game.options.reward_achievements,
            hotbar_inventory: game.options.hotbar_inventory,
            hotbar_hub_clock: game.options.hotbar_hub_clock,
            player_kick_idle: game.options.player_kick_idle,
            staff_only: game.options.staff_only,
            whitelist: game.options.whitelist,
            resource_pack: game.options.resource_pack,
            region: game.options.region,
            team_server_key: game.options.team_server.and_then(|serv| {
                GAME_TO_SERVER_PREFIX
                    .get(&serv)
                    .cloned()
                    .and_then(|g| Some(g.to_string()))
            }),
            portal_top_corner_location: game.options.portal_top_corner_location,
            portal_bottom_corner_location: game.options.portal_bottom_corner_location,
            npc_name: game.options.npc_name,
        }
    }
}

impl TryFrom<&str> for ServerGroup {
    type Error = ServerGroupParsingError;
    fn try_from(group: &str) -> Result<Self, Self::Error> {
        Self::get_server_group(&format!("servergroups.{}", group))
    }
}

impl TryFrom<HashMap<String, String>> for ServerGroup {
    type Error = ServerGroupParsingError;
    fn try_from(map: HashMap<String, String>) -> Result<Self, Self::Error> {
        if map.is_empty() {
            return Err(ServerGroupParsingError::new(
                "ServerGroup not found.".into(),
            ));
        }
        let name = map
            .get("name")
            .ok_or(ServerGroupParsingError::new(
                "ServerGroup's name could not be found".into(),
            ))?
            .to_string();
        let prefix = name.clone();
        assert_eq!(parse_value(&prefix, &map, "prefix")?, prefix);
        let server_group = Self {
            name,
            prefix: prefix.clone(),
            ram: parse_u16(&prefix, &map, "ram")?,
            cpu: parse_u8(&prefix, &map, "cpu")?,
            total_servers: parse_u8(&prefix, &map, "totalServers")?,
            joinable_servers: parse_u8(&prefix, &map, "joinableServers")?,
            port_section: parse_u16(&prefix, &map, "portSection")?,
            uptimes: parse_optional_str(&map, "uptimes")?,
            arcade_group: parse_bool_or_default(&prefix, &map, "arcadeGroup")?,
            world_zip: parse_value(&prefix, &map, "worldZip")?,
            plugin: parse_value(&prefix, &map, "plugin")?,
            config_path: parse_value(&prefix, &map, "configPath")?,
            host: parse_optional_str(&map, "host")?,
            min_players: parse_u8(&prefix, &map, "minPlayers")?,
            max_players: parse_u8(&prefix, &map, "maxPlayers")?,
            pvp: parse_bool_or_default(&prefix, &map, "pvp")?,
            tournament: parse_bool_or_default(&prefix, &map, "tournament")?,
            tournament_points: parse_bool_or_default(&prefix, &map, "tournamentPoints")?,
            hard_max_player_cap: parse_bool_or_default(&prefix, &map, "hardMaxPlayerCap")?,
            games: parse_optional_str(&map, "games")?,
            modes: parse_optional_str(&map, "modes")?,
            booster_group: parse_optional_str(&map, "boosterGroup")?,
            server_type: parse_value(&prefix, &map, "serverType")?,
            add_no_cheat: parse_bool_or_default(&prefix, &map, "addNoCheat")?,
            add_world_edit: parse_bool_or_default(&prefix, &map, "addWorldEdit")?,
            team_rejoin: parse_bool_or_default(&prefix, &map, "teamRejoin")?,
            team_auto_join: parse_bool_or_default(&prefix, &map, "teamAutoJoin")?,
            team_force_balance: parse_bool_or_default(&prefix, &map, "teamForceBalance")?,
            game_auto_start: parse_bool_or_default(&prefix, &map, "gameAutoStart")?,
            game_timeout: parse_bool_or_default(&prefix, &map, "gameTimeout")?,
            game_voting: parse_bool_or_default(&prefix, &map, "gameVoting")?,
            map_voting: parse_bool_or_default(&prefix, &map, "mapVoting")?,
            reward_gems: parse_bool_or_default(&prefix, &map, "rewardGems")?,
            reward_items: parse_bool_or_default(&prefix, &map, "rewardItems")?,
            reward_stats: parse_bool_or_default(&prefix, &map, "rewardStats")?,
            reward_achievements: parse_bool_or_default(&prefix, &map, "rewardAchievements")?,
            hotbar_inventory: parse_bool_or_default(&prefix, &map, "hotbarInventory")?,
            hotbar_hub_clock: parse_bool_or_default(&prefix, &map, "hotbarHubClock")?,
            player_kick_idle: parse_bool_or_default(&prefix, &map, "playerKickIdle")?,
            staff_only: parse_bool_or_default(&prefix, &map, "staffOnly")?,
            whitelist: parse_bool_or_default(&prefix, &map, "whitelist")?,
            resource_pack: parse_optional_str(&map, "resourcePack")?,
            region: Region::try_from(parse_value(&prefix, &map, "region").unwrap_or("US".into()))
                .map_err(|err| {
                ServerGroupParsingError::new(format!(
                    "servergroups.{} {:?}: {:?}",
                    &prefix, "region", err
                ))
            })?,
            team_server_key: parse_optional_str(&map, "teamServerKey")?,
            portal_bottom_corner_location: parse_optional_str(&map, "portalBottomCornerLocation")?,
            portal_top_corner_location: parse_optional_str(&map, "portalTopCornerLocation")?,
            npc_name: parse_optional_str(&map, "npcName")?,
        };
        Ok(server_group)
    }
}

impl From<ServerGroup> for HashMap<String, String> {
    fn from(group: ServerGroup) -> Self {
        HashMap::from([
            ("name".into(), group.name),
            ("prefix".into(), group.prefix),
            ("ram".into(), group.ram.to_string()),
            ("cpu".into(), group.cpu.to_string()),
            ("totalServers".into(), group.total_servers.to_string()),
            ("joinableServers".into(), group.joinable_servers.to_string()),
            ("portSection".into(), group.port_section.to_string()),
            ("uptimes".into(), group.uptimes.unwrap_or(String::new())),
            ("arcadeGroup".into(), group.arcade_group.to_string()),
            ("worldZip".into(), group.world_zip),
            ("plugin".into(), group.plugin),
            ("configPath".into(), group.config_path),
            ("host".into(), group.host.unwrap_or(String::new())),
            ("minPlayers".into(), group.min_players.to_string()),
            ("maxPlayers".into(), group.max_players.to_string()),
            ("pvp".into(), group.pvp.to_string()),
            ("tournament".into(), group.tournament.to_string()),
            (
                "tournamentPoints".into(),
                group.tournament_points.to_string(),
            ),
            (
                "hardMaxPlayerCap".into(),
                group.hard_max_player_cap.to_string(),
            ),
            ("games".into(), group.games.unwrap_or(String::new())),
            ("modes".into(), group.modes.unwrap_or(String::new())),
            (
                "boosterGroup".into(),
                group.booster_group.unwrap_or(String::new()),
            ),
            ("serverType".into(), group.server_type),
            ("addNoCheat".into(), group.add_no_cheat.to_string()),
            ("addWorldEdit".into(), group.add_world_edit.to_string()),
            ("teamRejoin".into(), group.team_rejoin.to_string()),
            ("teamAutoJoin".into(), group.team_auto_join.to_string()),
            (
                "teamForceBalance".into(),
                group.team_force_balance.to_string(),
            ),
            ("gameAutoStart".into(), group.game_auto_start.to_string()),
            ("gameTimeout".into(), group.game_timeout.to_string()),
            ("gameVoting".into(), group.game_voting.to_string()),
            ("mapVoting".into(), group.map_voting.to_string()),
            ("rewardGems".into(), group.reward_gems.to_string()),
            ("rewardItems".into(), group.reward_items.to_string()),
            ("rewardStats".into(), group.reward_stats.to_string()),
            (
                "rewardAchievements".into(),
                group.reward_achievements.to_string(),
            ),
            ("hotbarInventory".into(), group.hotbar_inventory.to_string()),
            ("hotbarHubClock".into(), group.hotbar_hub_clock.to_string()),
            ("playerKickIdle".into(), group.player_kick_idle.to_string()),
            ("staffOnly".into(), group.staff_only.to_string()),
            ("whitelist".into(), group.whitelist.to_string()),
            (
                "resourcePack".into(),
                group.resource_pack.unwrap_or(String::new()),
            ),
            ("region".into(), group.region.into()),
            (
                "teamServerKey".into(),
                group.team_server_key.unwrap_or(String::new()),
            ),
            (
                "portalBottomCornerLocation".into(),
                group.portal_bottom_corner_location.unwrap_or(String::new()),
            ),
            (
                "portalTopCornerLocation".into(),
                group.portal_top_corner_location.unwrap_or(String::new()),
            ),
            ("npcName".into(), group.npc_name.unwrap_or(String::new())),
        ])
    }
}

impl ServerGroup {
    pub fn create(&self) -> Result<(), redis::RedisError> {
        let config: Config = Config::get_config();
        let mut conn = connect(&config);
        let redis_key: String = format!("servergroups.{}", self.prefix);
        let sg = Self::get_server_group(&redis_key).ok();
        if sg.is_some() {
            // exists in redis already
            return Ok(());
        }
        let params: HashMap<String, String> = self.clone().into();
        let _: () = redis::cmd("HSET")
            .arg(redis_key)
            .arg(params)
            .query(&mut conn)?;
        Ok(())
    }

    pub fn get_server_group(redis_key: &String) -> Result<ServerGroup, ServerGroupParsingError> {
        let config: Config = Config::get_config();
        let mut conn = connect(&config);
        let redis_data: HashMap<String, String> = redis::cmd("HGETALL")
            .arg(redis_key)
            .query(&mut conn)
            .map_err(|_| {
                ServerGroupParsingError::new(
                    "Redis data for ServerGroup could not be retrieved".into(),
                )
            })?;
        Ok(Self::try_from(redis_data)?)
    }

    pub fn get_server_groups() -> Result<Vec<ServerGroup>, ServerGroupParsingError> {
        let config: Config = Config::get_config();
        let mut conn = connect(&config);
        let server_groups: Vec<String> = redis::cmd("KEYS")
            .arg("servergroups.*")
            .query(&mut conn)
            .map_err(|_| {
                ServerGroupParsingError::new(
                    "Redis data for ServerGroup could not be retrieved. ServerGroup iteration failed."
                        .into(),
                )
            })?;
        server_groups
            .iter()
            .map(|sg| Self::get_server_group(sg))
            .collect()
    }

    pub fn get_all_port_sections() -> Result<Vec<u16>, ServerGroupParsingError> {
        let server_groups: Vec<ServerGroup> = Self::get_server_groups()?;
        let ports: Vec<u16> = server_groups
            .iter()
            .map(|group| group.port_section)
            .collect();
        Ok(ports)
    }
}
