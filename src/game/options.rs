use std::str::FromStr;

use rand::Rng;
use strum::IntoEnumIterator;

use crate::{
    error::parsing_error::ServerGroupParsingError, region::Region,
    server::server_group::ServerGroup,
};

use super::{
    booster_group::BoosterGroup,
    r#type::GameType,
    utils::{
        CUSTOM_GAME_OPTIONS, GAME_TO_BOOSTER_GROUP, GAME_TO_NPC, GAME_TO_PLAYER_COUNT,
        GAME_TO_SERVER_PREFIX, GAME_TO_TEAM_SERVER, SERVER_PREFIX_TO_GAME,
    },
};

#[derive(Clone, Debug)]
pub struct GameOptions {
    pub prefix: String,
    pub staff_only: bool,
    pub whitelist: bool,
    pub host: Option<String>,
    pub min_players: u8,
    pub max_players: u8,
    pub port_section: u16,
    pub arcade_group: bool,
    pub world_zip: String,
    pub plugin: String,
    pub config_path: String,
    pub pvp: bool,
    pub tournament: bool,
    pub tournament_points: bool,
    pub games: Option<String>,
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
    pub team_server: Option<GameType>,
    pub booster_group: Option<BoosterGroup>,
    pub npc_name: Option<String>,
    pub resource_pack: Option<String>,
    pub region: Region,
    pub portal_bottom_corner_location: Option<String>,
    pub portal_top_corner_location: Option<String>,
}

impl TryFrom<GameType> for GameOptions {
    type Error = ServerGroupParsingError;
    fn try_from(game: GameType) -> Result<Self, Self::Error> {
        let binding = Self::load_from_cache(&game);
        let cached: Option<&ServerGroup> = binding.as_ref();
        if let Some(options) = CUSTOM_GAME_OPTIONS.get(&game) {
            if cached.is_none() {
                let mut new = options.clone();
                new.port_section = Self::rnd_port()?;
                return Ok(new);
            }
        }
        let (min_players, max_players) = cached.map_or(
            GAME_TO_PLAYER_COUNT.get(&game).cloned().unwrap_or((8, 16)),
            |data| (data.min_players, data.max_players),
        );
        Ok(Self {
            prefix: GAME_TO_SERVER_PREFIX
                .get(&game)
                .cloned()
                .unwrap_or_default()
                .into(),
            staff_only: cached.map_or(false, |data| data.staff_only),
            whitelist: cached.map_or(false, |data| data.whitelist),
            host: cached
                .and_then(|data| data.host.clone())
                .filter(|x| !x.is_empty()),
            min_players,
            max_players,
            port_section: cached.map_or(Self::rnd_port()?, |data| data.port_section), // makes unique
            arcade_group: cached.map_or(true, |data| data.arcade_group),
            world_zip: cached.map_or("arcade.zip".into(), |data| data.world_zip.clone()),
            plugin: cached.map_or("Arcade.jar".into(), |data| data.plugin.clone()),
            config_path: cached.map_or("plugins/Arcade".into(), |data| data.config_path.clone()),
            pvp: cached.map_or(true, |data| data.pvp),
            tournament: cached.map_or(false, |data| data.tournament),
            tournament_points: cached.map_or(false, |data| data.tournament_points),
            games: cached.map_or(
                match Some(game) {
                    Some(GameType::MixedArcade) => Some(
                        GameType::iter()
                            .take(7)
                            .fold(String::new(), |a, b| {
                                if a.is_empty() {
                                    b.to_string()
                                } else {
                                    format!("{},{}", a, b)
                                }
                            })
                            .to_string(),
                    ),
                    Some(g) => Some(g.to_string()),
                    None => None,
                },
                |data| data.games.clone().filter(|x| !x.is_empty() && x != "null"),
            ),
            server_type: cached.map_or("Minigames".into(), |data| data.server_type.clone()),
            add_no_cheat: cached.map_or(true, |data| data.add_no_cheat),
            add_world_edit: cached.map_or(false, |data| data.add_world_edit),
            team_rejoin: cached.map_or(false, |data| data.team_rejoin),
            team_auto_join: cached.map_or(true, |data| data.team_auto_join),
            team_force_balance: cached.map_or(false, |data| data.team_force_balance),
            game_auto_start: cached.map_or(true, |data| data.game_auto_start),
            game_timeout: cached.map_or(true, |data| data.game_timeout),
            game_voting: cached.map_or(false, |data| data.game_voting),
            map_voting: cached.map_or(true, |data| data.map_voting),
            reward_gems: cached.map_or(true, |data| data.reward_gems),
            reward_items: cached.map_or(true, |data| data.reward_items),
            reward_stats: cached.map_or(true, |data| data.reward_stats),
            reward_achievements: cached.map_or(true, |data| data.reward_achievements),
            hotbar_inventory: cached.map_or(true, |data| data.hotbar_inventory),
            hotbar_hub_clock: cached.map_or(true, |data| data.hotbar_hub_clock),
            player_kick_idle: cached.map_or(true, |data| data.player_kick_idle),
            team_server: cached.map_or(GAME_TO_TEAM_SERVER.get(&game).cloned(), |data| {
                SERVER_PREFIX_TO_GAME
                    .get(&data.team_server_key.clone()?.as_ref())
                    .cloned()
            }),
            booster_group: cached.map_or(GAME_TO_BOOSTER_GROUP.get(&game).cloned(), |data| {
                BoosterGroup::from_str(data.booster_group.clone()?.as_ref()).ok()
            }),
            npc_name: cached.map_or(
                GAME_TO_NPC.get(&game).cloned().map(|x| x.to_string()),
                |data| data.npc_name.clone().filter(|x| !x.is_empty()),
            ),
            resource_pack: cached
                .and_then(|data| data.resource_pack.clone())
                .filter(|x| !x.is_empty()),
            region: cached.map_or(Region::US, |data| data.region.clone()),
            portal_bottom_corner_location: cached
                .and_then(|data| data.portal_bottom_corner_location.clone())
                .filter(|x| !x.is_empty()),
            portal_top_corner_location: cached
                .and_then(|data| data.portal_top_corner_location.clone())
                .filter(|x| !x.is_empty()),
        })
    }
}

impl GameOptions {
    pub fn check_port_section_conflicts(port_section: u16, cached_ports: &Vec<u16>) -> bool {
        //! Checks if new port section conflicts with other port sections in cache.
        //! Each port section is unique to their ServerGroup.
        //! A port section holds 10 values where a certain server instances's port can be made from.
        //! A server instance's port can be anything ten above the current port section of its servergroup.
        cached_ports.iter().any(|&cached_port| {
            (port_section < cached_port && cached_port <= port_section + 10) // cache conflicts with NEW
            || (cached_port < port_section && port_section <= cached_port + 10) // OR NEW conflicts with cache
            || (cached_port == port_section) // they're the same
        })
    }

    fn rnd_port() -> Result<u16, ServerGroupParsingError> {
        //! Returns non-conflicting port section
        let mut rng = rand::thread_rng();
        let port_sections: Vec<u16> = ServerGroup::get_all_port_sections()?;
        let mut port_section: u16 = rng.gen_range(25565..26001);
        while Self::check_port_section_conflicts(port_section, &port_sections)
        // port section conflict (either can be 10 above the other)
        {
            port_section = rng.gen_range(25565..26001);
        }
        Ok(port_section)
    }

    fn load_from_cache(game: &GameType) -> Option<ServerGroup> {
        let prefix = GAME_TO_SERVER_PREFIX.get(game).cloned()?;
        ServerGroup::get_server_group(&format!("servergroups.{}", prefix)).ok()
    }
}
