use rand::rngs::ThreadRng;
use rand::Rng;
use redis::RedisError;

use crate::config::models::Config;
use crate::context_manager::ContextManager;
use crate::error::parsing_error::ServerGroupParsingError;
use crate::game::options::GameOptions;
use crate::game::utils::GAME_TO_SERVER_PREFIX;
use crate::game::Game;
use crate::region::Region;
use std::collections::HashMap;

use super::minecraft::MinecraftServer;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

impl From<ServerGroupParsingError> for RedisError {
    fn from(err: ServerGroupParsingError) -> Self {
        (
            redis::ErrorKind::ParseError,
            "ServerGroup parsing error",
            err.msg,
        )
            .into()
    }
}

impl ServerGroup {
    pub fn from_game(game: Game) -> Self {
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

    pub fn from_hashmap(map: HashMap<String, String>) -> Result<Self, ServerGroupParsingError> {
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

    /// Loads from cache or default
    pub fn from_str(
        group: &str,
        ctx: &mut ContextManager,
    ) -> Result<Self, ServerGroupParsingError> {
        Self::get_server_group(&format!("servergroups.{}", group), ctx)
    }

    pub fn to_hashmap(&self) -> HashMap<String, String> {
        HashMap::from([
            ("name".into(), self.name.clone()),
            ("prefix".into(), self.prefix.clone()),
            ("ram".into(), self.ram.to_string()),
            ("cpu".into(), self.cpu.to_string()),
            ("totalServers".into(), self.total_servers.to_string()),
            ("joinableServers".into(), self.joinable_servers.to_string()),
            ("portSection".into(), self.port_section.to_string()),
            (
                "uptimes".into(),
                self.uptimes.clone().unwrap_or(String::new()),
            ),
            ("arcadeGroup".into(), self.arcade_group.to_string()),
            ("worldZip".into(), self.world_zip.clone()),
            ("plugin".into(), self.plugin.clone()),
            ("configPath".into(), self.config_path.clone()),
            ("host".into(), self.host.clone().unwrap_or(String::new())),
            ("minPlayers".into(), self.min_players.to_string()),
            ("maxPlayers".into(), self.max_players.to_string()),
            ("pvp".into(), self.pvp.to_string()),
            ("tournament".into(), self.tournament.to_string()),
            (
                "tournamentPoints".into(),
                self.tournament_points.to_string(),
            ),
            (
                "hardMaxPlayerCap".into(),
                self.hard_max_player_cap.to_string(),
            ),
            ("games".into(), self.games.clone().unwrap_or(String::new())),
            ("modes".into(), self.modes.clone().unwrap_or(String::new())),
            (
                "boosterGroup".into(),
                self.booster_group.clone().unwrap_or(String::new()),
            ),
            ("serverType".into(), self.server_type.clone()),
            ("addNoCheat".into(), self.add_no_cheat.to_string()),
            ("addWorldEdit".into(), self.add_world_edit.to_string()),
            ("teamRejoin".into(), self.team_rejoin.to_string()),
            ("teamAutoJoin".into(), self.team_auto_join.to_string()),
            (
                "teamForceBalance".into(),
                self.team_force_balance.to_string(),
            ),
            ("gameAutoStart".into(), self.game_auto_start.to_string()),
            ("gameTimeout".into(), self.game_timeout.to_string()),
            ("gameVoting".into(), self.game_voting.to_string()),
            ("mapVoting".into(), self.map_voting.to_string()),
            ("rewardGems".into(), self.reward_gems.to_string()),
            ("rewardItems".into(), self.reward_items.to_string()),
            ("rewardStats".into(), self.reward_stats.to_string()),
            (
                "rewardAchievements".into(),
                self.reward_achievements.to_string(),
            ),
            ("hotbarInventory".into(), self.hotbar_inventory.to_string()),
            ("hotbarHubClock".into(), self.hotbar_hub_clock.to_string()),
            ("playerKickIdle".into(), self.player_kick_idle.to_string()),
            ("staffOnly".into(), self.staff_only.to_string()),
            ("whitelist".into(), self.whitelist.to_string()),
            (
                "resourcePack".into(),
                self.resource_pack.clone().unwrap_or(String::new()),
            ),
            ("region".into(), self.region.clone().to_string()),
            (
                "teamServerKey".into(),
                self.team_server_key.clone().unwrap_or(String::new()),
            ),
            (
                "portalBottomCornerLocation".into(),
                self.portal_bottom_corner_location
                    .clone()
                    .unwrap_or(String::new()),
            ),
            (
                "portalTopCornerLocation".into(),
                self.portal_top_corner_location
                    .clone()
                    .unwrap_or(String::new()),
            ),
            (
                "npcName".into(),
                self.npc_name.clone().unwrap_or(String::new()),
            ),
        ])
    }

    pub fn load_existing_cache(&mut self, ctx: &mut ContextManager) -> () {
        //! ServerGroup returns to cached redis state if exists.
        let redis_key: String = format!("servergroups.{}", self.prefix);
        let cached: Option<ServerGroup> = Self::get_server_group(&redis_key, ctx).ok();
        if cached.is_none() {
            ()
        }
        *self = cached.unwrap().clone();
        ()
    }

    pub fn is_cached(&self, ctx: &mut ContextManager) -> bool {
        //! Returns if ServerGroup was cached in redis.
        let redis_key: String = format!("servergroups.{}", self.prefix);
        Self::get_server_group(&redis_key, ctx).is_ok()
    }

    pub fn delete(&self, ctx: &mut ContextManager) -> Result<(), redis::RedisError> {
        //! Deletes ServerGroup from cache.
        let redis_key: String = format!("servergroups.{}", self.prefix);
        if self.is_cached(ctx) {
            let _: () = redis::cmd("DEL")
                .arg(redis_key)
                .query(ctx.get_connection())?;
        }
        let _: () = redis::cmd("SREM")
            .arg("servergroups")
            .arg(&self.prefix)
            .query(ctx.get_connection())?;
        Ok(())
    }

    pub fn eliminate_port_collisions(
        &mut self,
        ctx: &mut ContextManager,
    ) -> Result<(), ServerGroupParsingError> {
        //! Eliminates port collisions between `self` and cached `ServerGroup`s by generating a new
        //! port section.
        //! (Call this function before caching)
        self.reset_port_section_if_invalid(ctx).map_err(|err| {
            ServerGroupParsingError::new(format!(
                "Error while executing `eliminate_port_collisions` in ServerGroup (could not reset port): {:?}",
                err
            ))
        })?;
        Ok(())
    }

    fn get_port_section_is_invalid(
        &self,
        ctx: &mut ContextManager,
    ) -> Result<bool, ServerGroupParsingError> {
        //! Returns `true` if port section conflicts with another group's cached port section, otherwise `false`.
        //! Raises ServerGroupParsingError if there are issues while fetching existing port_sections.
        Ok(GameOptions::check_port_section_conflicts(
            self.port_section,
            &self.get_all_other_port_sections(ctx)?,
        ))
    }

    fn get_random_port_section(&mut self, rng: &mut ThreadRng) -> () {
        //! Generates any random port from 25566 to 25600.
        self.port_section = rng.gen_range(25566..26001);
    }

    fn reset_port_section_if_invalid(
        &mut self,
        ctx: &mut ContextManager,
    ) -> Result<(), ServerGroupParsingError> {
        //! Resets port section if it conflicts with another group's cached port section.
        let mut rng = rand::thread_rng();
        while self.get_port_section_is_invalid(ctx)? {
            self.get_random_port_section(&mut rng);
        }
        Ok(())
    }

    fn find_port_conflicts(
        &mut self,
        ctx: &mut ContextManager,
    ) -> Result<Vec<String>, ServerGroupParsingError> {
        //! Filters for servergroups with conflicting ports to self.
        //! Returns a vec of their names.
        let server_groups: Vec<ServerGroup> = Self::get_server_groups(ctx)?
            .into_iter()
            .filter(|sg| sg.name != self.name)
            .collect();
        Ok(server_groups
            .into_iter()
            .filter_map(|sg| {
                Some(GameOptions::get_if_port_section_conflict(
                    self.port_section,
                    sg.port_section,
                ))
                .filter(|&x| x)
                .map(|_| sg.name)
            })
            .collect())
    }

    fn get_all_other_port_sections(
        &self,
        ctx: &mut ContextManager,
    ) -> Result<Vec<u16>, ServerGroupParsingError> {
        //! Returns a vec of cached port sections that don't include self (even if it is cached).
        let server_groups: Vec<ServerGroup> = Self::get_server_groups(ctx)?;
        Ok(server_groups
            .into_iter()
            .filter_map(|sg| Some(sg.name != self.name).map(|_| sg.port_section))
            .collect())
    }

    pub fn create(&mut self, ctx: &mut ContextManager) -> Result<(), redis::RedisError> {
        let redis_key: String = format!("servergroups.{}", self.prefix);
        let sg = Self::get_server_group(&redis_key, ctx).ok();
        if sg.is_some() {
            // if exists in redis already
            let _: () = redis::cmd("SADD") // even if it exists in set
                .arg("servergroups")
                .arg(&self.prefix)
                .query(ctx.get_connection())?;
            return Ok(());
        }
        self.eliminate_port_collisions(ctx)?; // no more conflicting ports
        let params: HashMap<String, String> = self.to_hashmap();
        let _: () = redis::cmd("HSET")
            .arg(redis_key)
            .arg(params)
            .query(ctx.get_connection())?;
        let _: () = redis::cmd("SADD")
            .arg("servergroups")
            .arg(&self.prefix)
            .query(ctx.get_connection())?;
        Ok(())
    }

    pub fn get_server_group(
        redis_key: &String,
        ctx: &mut ContextManager,
    ) -> Result<ServerGroup, ServerGroupParsingError> {
        let config: Config = Config::get_config();
        let mut conn = ctx.get_connection();
        let redis_data: HashMap<String, String> = redis::cmd("HGETALL")
            .arg(redis_key)
            .query(&mut conn)
            .map_err(|_| {
                ServerGroupParsingError::new(
                    "Redis data for ServerGroup could not be retrieved".into(),
                )
            })?;
        Ok(Self::from_hashmap(redis_data)?)
    }

    pub fn get_server_groups(
        ctx: &mut ContextManager,
    ) -> Result<Vec<ServerGroup>, ServerGroupParsingError> {
        let mut conn = ctx.get_connection();
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
            .map(|sg| Self::get_server_group(sg, ctx))
            .collect()
    }

    pub fn get_all_port_sections(
        ctx: &mut ContextManager,
    ) -> Result<Vec<u16>, ServerGroupParsingError> {
        let server_groups: Vec<ServerGroup> = Self::get_server_groups(ctx)?;
        let ports: Vec<u16> = server_groups
            .iter()
            .map(|group| group.port_section)
            .collect();
        Ok(ports)
    }
}
