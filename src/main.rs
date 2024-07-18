mod config;
mod error;
mod game;
mod redis;
mod region;
mod server;

use game::Game;

use crate::{
    error::parsing_error::ServerGroupParsingError,
    game::r#type::GameType,
    region::Region,
    server::{minecraft::MinecraftServer, server_group::ServerGroup},
};

#[allow(dead_code)]
enum GenericServer {
    Lobby,
    ClansHub,
    BetaHub,
}

fn main() {
    let game: Result<Game, ServerGroupParsingError> = Game::try_from(GameType::ClansHub);
    let clans_hub: ServerGroup = ServerGroup::from(game.unwrap());
    let server_statuses = MinecraftServer::get_all();
    dbg!(clans_hub);
    dbg!(server_statuses);
    //let ports: Result<Vec<u16>, ServerGroupParsingError> = ServerGroup::get_all_port_sections();
    //dbg!(ports);
}
