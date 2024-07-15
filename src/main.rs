use lazy_static::lazy_static;
use rand::Rng;
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::Read,
    str::FromStr,
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use serde::{Deserialize, Serialize};

enum GenericServer {
    Lobby,
    ClansHub,
    BetaHub,
}

#[derive(Debug)]
struct ServerGroupParsingError {
    msg: String,
}

impl Display for ServerGroupParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl Error for ServerGroupParsingError {}

impl ServerGroupParsingError {
    fn new(msg: String) -> Self {
        ServerGroupParsingError { msg }
    }
}

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
    staff_only: bool,
    whitelist: bool,
    host: Option<String>,
    min_players: u8,
    max_players: u8,
    port_section: u16,
    arcade_group: bool,
    world_zip: String,
    plugin: String,
    config_path: String,
    pvp: bool,
    tournament: bool,
    tournament_points: bool,
    games: Option<String>,
    server_type: String,
    add_no_cheat: bool,
    add_world_edit: bool,
    team_rejoin: bool,
    team_auto_join: bool,
    team_force_balance: bool,
    game_auto_start: bool,
    game_timeout: bool,
    game_voting: bool,
    map_voting: bool,
    reward_gems: bool,
    reward_items: bool,
    reward_stats: bool,
    reward_achievements: bool,
    hotbar_inventory: bool,
    hotbar_hub_clock: bool,
    player_kick_idle: bool,
    team_server: Option<GameType>,
    booster_group: Option<BoosterGroup>,
    npc_name: Option<String>,
    resource_pack: Option<String>,
    region: Region,
    portal_bottom_corner_location: Option<String>,
    portal_top_corner_location: Option<String>,
}

impl GameOptions {
    fn from(game: &GameType) -> Self {
        let binding = Self::load_from_cache(game);
        let cached = binding.as_ref();
        let (min_players, max_players) = cached.map_or(
            GAME_TO_PLAYER_COUNT.get(game).cloned().unwrap_or((8, 16)),
            |data| (data.min_players, data.max_players),
        );
        Self {
            prefix: GAME_TO_SERVER_PREFIX
                .get(game)
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
            port_section: cached.map_or(Self::rnd_port(), |data| data.port_section), // make unique
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
            team_server: cached.map_or(GAME_TO_TEAM_SERVER.get(game).cloned(), |data| {
                SERVER_PREFIX_TO_GAME
                    .get(&data.team_server_key.clone()?.as_ref())
                    .cloned()
            }),
            booster_group: cached.map_or(GAME_TO_BOOSTER_GROUP.get(game).cloned(), |data| {
                BoosterGroup::from_str(data.booster_group.clone()?.as_ref()).ok()
            }),
            npc_name: cached.map_or(
                GAME_TO_NPC.get(game).cloned().map(|x| x.to_string()),
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
        }
    }

    fn rnd_port() -> u16 {
        // returns non-conflicting port section
        let mut rng = rand::thread_rng();
        let ports = get_all_port_sections().unwrap_or(Vec::new());
        let mut port: u16 = rng.gen_range(25565..26001);
        while ports.iter().any(|&cached_port| {
            (port < cached_port && cached_port < port + 10) // cache conflicts with NEW.
                | (cached_port < port && port <= cached_port + 10) // OR NEW conflicts with cache
        })
        // port section conflict (either can be 10 above the other)
        {
            port = rng.gen_range(25565..26001);
        }
        port
    }

    fn load_from_cache(game: &GameType) -> Option<ServerGroup> {
        let prefix = GAME_TO_SERVER_PREFIX.get(game).cloned()?;
        get_server_group(&format!("servergroups.{}", prefix)).ok()
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Hash, EnumString, EnumIter)]
enum GameType {
    // synonymous for GameDisplay in Mineplex's code
    Micro,
    MixedArcade,
    Cards,
    Draw,
    Build,
    BuildMavericks,
    Tug,
    TurfWars,
    UHC,
    UHCSolo,
    UHCSoloSpeed,
    UHCTeamsSpeed,
    SpeedBuilders,
    Valentines,
    Skyfall,
    SkyfallTeams,
    HideSeek,
    CakeWarsDuos,
    CakeWars4,
    SurvivalGames,
    SurvivalGamesTeams,
    Skywars,
    SkywarsTeams,
    MonsterMaze,
    MonsterLeague,
    Bridges,
    MineStrike,
    Smash,
    SmashDominate,
    SmashTeams,
    SmashTraining,
    ChampionsCTF,
    BouncyBalls,
    Gladiators,
    TypeWars,
    ChampionsDominate,
    ChampionsTDM,
    Christmas,
    ChristmasNew,
    Clans,
    ClansHub,
    BaconBrawl,
    Barbarians,
    Basketball,
    QuiverPayload,
    StrikeGames,
    AlienInvasion,
    MOBA,
    MOBATraining,
    BattleRoyale,
    BossBattles,
    BawkBawkBattles,
    Brawl, // Event
    CastleAssault,
    CastleAssaultTDM,
    CastleSiege,
    DeathTag,
    DragonEscape,
    DragonEscapeTeams,
    DragonRiders,
    Dragons,
    Event,
    Evolution,
    ElytraRings,
    Gravity,
    GemHunters,
    Halloween,
    Halloween2016,
    HoleInTheWall,
    Horse,
    MilkCow,
    NanoGames,
    Lobbers,
    MinecraftLeague,
    OldMineWare,
    Paintball,
    Quiver,
    QuiverTeams,
    Runner,
    SearchAndDestroy,
    Sheep,
    Snake,
    SneakyAssassins,
    SnowFight,
    Spleef,
    SpleefTeams,
    SquidShooter,
    Stacker,
    WitherAssault,
    Wizards,
    ZombieSurvival,
}

struct Games {
    data: Vec<GameType>,
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
        (GameType::ChampionsDominate, BoosterGroup::Champions),
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
        (GameType::ChampionsDominate, (8, 10)),
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
        (GameType::ChampionsDominate, "DOM"),
        (GameType::ChampionsCTF, "CTF"),
        (GameType::Clans, "Clans"),
        (GameType::ClansHub, "ClansHub"),
    ]);
    static ref SERVER_PREFIX_TO_GAME: HashMap<&'static str, GameType> = HashMap::from([
        ("MB", GameType::Micro),
        ("MIN", GameType::MixedArcade),
        ("DMT", GameType::Draw),
        ("BLD", GameType::Build),
        ("TF", GameType::TurfWars),
        ("SB", GameType::SpeedBuilders),
        ("BH", GameType::HideSeek),
        ("CW2", GameType::CakeWarsDuos),
        ("CW4", GameType::CakeWars4),
        ("HG", GameType::SurvivalGames),
        ("SG2", GameType::SurvivalGamesTeams),
        ("SKY", GameType::Skywars),
        ("SKY2", GameType::SkywarsTeams),
        ("BR", GameType::Bridges),
        ("MS", GameType::MineStrike),
        ("SSM", GameType::Smash),
        ("SSM2", GameType::SmashTeams),
        ("DOM", GameType::ChampionsDominate),
        ("CTF", GameType::ChampionsCTF),
        ("Clans", GameType::Clans),
        ("ClansHub", GameType::ClansHub),
    ]);
    static ref GAME_TO_TEAM_SERVER: HashMap<GameType, GameType> = HashMap::from([
        (GameType::Skywars, GameType::SkywarsTeams),
        (GameType::SurvivalGames, GameType::SurvivalGamesTeams),
        (GameType::Smash, GameType::SmashTeams),
        (GameType::ChampionsDominate, GameType::ChampionsCTF),
        (GameType::CakeWars4, GameType::CakeWarsDuos),
    ]);
}

#[derive(Clone, Copy, Debug, Display, EnumString)]
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

#[derive(Clone, Debug)]
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

impl TryFrom<String> for Region {
    type Error = ServerGroupParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "US" | "" => Ok(Region::US),
            "EU" => Ok(Region::EU),
            "ALL" => Ok(Region::ALL),
            _ => Err(ServerGroupParsingError::new(
                "Region could not be parsed.".into(),
            )),
        }
    }
}

#[derive(Debug)]
struct ServerGroup {
    name: String,
    prefix: String,
    ram: u16,
    cpu: u8,
    total_servers: u8,
    joinable_servers: u8,
    port_section: u16,
    uptimes: Option<String>,
    arcade_group: bool,
    world_zip: String,
    plugin: String,
    config_path: String,
    host: Option<String>,
    min_players: u8,
    max_players: u8,
    pvp: bool,
    tournament: bool,
    tournament_points: bool,
    hard_max_player_cap: bool,
    games: Option<String>,
    modes: Option<String>,
    booster_group: Option<String>,
    server_type: String,
    add_no_cheat: bool,
    add_world_edit: bool,
    team_rejoin: bool,
    team_auto_join: bool,
    team_force_balance: bool,
    game_auto_start: bool,
    game_timeout: bool,
    game_voting: bool,
    map_voting: bool,
    reward_gems: bool,
    reward_items: bool,
    reward_stats: bool,
    reward_achievements: bool,
    hotbar_inventory: bool,
    hotbar_hub_clock: bool,
    player_kick_idle: bool,
    staff_only: bool,
    whitelist: bool,
    resource_pack: Option<String>,
    region: Region,
    team_server_key: Option<String>,
    portal_bottom_corner_location: Option<String>,
    portal_top_corner_location: Option<String>,
    npc_name: Option<String>,
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

fn parse_value<'a>(
    prefix: &String,
    map: &HashMap<String, String>,
    key: &'a str,
) -> Result<String, ServerGroupParsingError> {
    Ok(map
        .get(key)
        .ok_or(ServerGroupParsingError::new(format!(
            "servergroups.{}  {:?} could not be found.",
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
    Ok(map.get(key).cloned())
}

impl TryFrom<&str> for ServerGroup {
    type Error = ServerGroupParsingError;
    fn try_from(group: &str) -> Result<Self, Self::Error> {
        get_server_group(&format!("servergroups.{}", group))
    }
}

impl TryFrom<HashMap<String, String>> for ServerGroup {
    type Error = ServerGroupParsingError;
    fn try_from(map: HashMap<String, String>) -> Result<Self, Self::Error> {
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
            region: Region::try_from(parse_value(&prefix, &map, "region")?).map_err(|err| {
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

fn get_server_groups() -> Result<Vec<ServerGroup>, ServerGroupParsingError> {
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
        .map(|sg| get_server_group(sg))
        .collect()
}

fn get_all_port_sections() -> Result<Vec<u16>, ServerGroupParsingError> {
    let server_groups: Vec<ServerGroup> = get_server_groups()?;
    let ports: Vec<u16> = server_groups
        .iter()
        .map(|group| group.port_section)
        .collect();
    Ok(ports)
}

fn get_server_group(redis_key: &String) -> Result<ServerGroup, ServerGroupParsingError> {
    let config: Config = Config::get_config();
    let mut conn = connect(&config);
    let redis_data: HashMap<String, String> = redis::cmd("HGETALL")
        .arg(redis_key)
        .query(&mut conn)
        .map_err(|_| {
            ServerGroupParsingError::new("Redis data for ServerGroup could not be retrieved".into())
        })?;
    Ok(ServerGroup::try_from(redis_data)?)
}

fn main() {
    dbg!(ServerGroup::try_from("DOM").ok());
    //dbg!(ServerGroup::from(Game::from(&GameType::ChampionsDominate)));
    //let ports: Result<Vec<u16>, ServerGroupParsingError> = get_all_port_sections();
    //dbg!(ports);
}
