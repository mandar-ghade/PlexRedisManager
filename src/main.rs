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
    let lobby_server_status =
        MinecraftServer::get_server_status(&String::from("MIN-1"), &Region::US);
    dbg!(clans_hub);
    dbg!(lobby_server_status);
    //let ports: Result<Vec<u16>, ServerGroupParsingError> = ServerGroup::get_all_port_sections();
    //dbg!(ports);
}
