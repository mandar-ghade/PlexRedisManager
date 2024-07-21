mod config;
mod ctx_manager;
mod error;
mod game;
mod redis;
mod region;
mod server;

use game::Game;

use crate::{
    config::models::Config,
    error::parsing_error::ServerGroupParsingError,
    game::r#type::GameType,
    region::Region,
    server::{generic::GenericServer, minecraft::MinecraftServer, server_group::ServerGroup},
};

fn get_best_server_test(cfg: &mut Config, group: &ServerGroup) -> () {
    let dedicated_servers = &mut cfg.dedicated_servers;
    dbg!(&dedicated_servers);
    let next = dedicated_servers.get_best_dedicated_server(group);
    dbg!(&next);
    if let Some(dedi) = next {
        dedi.add_server(group);
        dbg!(dedi);
    }
    let next_available = dedicated_servers.get_best_dedicated_server(group); // new server found
    dbg!(&next_available);
}

fn main() {
    let game: Result<Game, ServerGroupParsingError> = Game::try_from(GameType::MixedArcade);
    let mut _mixed_arcade: ServerGroup = ServerGroup::from(game.unwrap());
    dbg!(&_mixed_arcade);
    //let mut _cfg = Config::get_config();
    //get_best_server_test(&mut cfg, &mixed_arcade);
    //dbg!(&_cfg);

    //let lobby = GenericServer::Lobby.to_server_group();
    //dbg!(&lobby);

    //let server_statuses = MinecraftServer::get_all();
}
