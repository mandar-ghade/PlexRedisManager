mod config;
mod context_manager;
mod error;
mod game;
mod region;
mod server;

use game::Game;

use crate::{
    config::models::Config,
    context_manager::ContextManager,
    error::parsing_error::ServerGroupParsingError,
    game::r#type::GameType,
    region::Region,
    server::{
        dedicated::server::DedicatedServer, generic::GenericServer, minecraft::MinecraftServer,
        server_group::ServerGroup,
    },
};

fn get_best_server_test(group: &ServerGroup, ctx: &mut ContextManager) -> () {
    let dedicated_servers = ctx.get_dedicated_servers();
    dbg!(&dedicated_servers);
    let next: Option<&mut DedicatedServer> = dedicated_servers.get_best_dedicated_server(group);
    let _ = next.map(|dedi| {
        dedi.add_server(group);
        dbg!(&dedi);
    });
    let next_available = dedicated_servers.get_best_dedicated_server(group); // new server found
    dbg!(&next_available);
}

fn main() {
    let mut ctx: ContextManager = ContextManager::new();
    let game: Game = Game::from_game_type(GameType::MixedArcade, &mut ctx)
        .expect("Could not convert from GameType to Game");
    let mixed_arcade = ServerGroup::from_game(game);
    dbg!(&mixed_arcade);
    //get_best_server_test(&mixed_arcade, &mut ctx)
}
