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
    let server_num = dedicated_servers.get_next_server_num(group);
    let next: Option<&mut DedicatedServer> = dedicated_servers.get_best_dedicated_server(group);
    let _ = next.map(|dedi| {
        let result = dedi.add_server(group, server_num);
        dbg!(result);
        dbg!(&dedi);
    });
    dbg!(&dedicated_servers);
    let next_available = dedicated_servers.get_best_dedicated_server(group); // new server found
    dbg!(&next_available);
}

fn main() {
    let mut ctx: ContextManager = ContextManager::new();
    //let mut lobby = GenericServer::Lobby
    //    .to_server_group(&mut ctx)
    //    .expect("Lobby expected.")
    //    .expect("Should've been Some(Lobby)");
    let arcade = ServerGroup::from_game(
        Game::from_game_type(GameType::MixedArcade, &mut ctx)
            .expect("Game type should've been found"),
    );
    dbg!(get_best_server_test(&arcade, &mut ctx))
    //let _ = lobby.create(&mut ctx);
    //dbg!(MinecraftServer::from_server_group(&arcade, &mut ctx));
    //dbg!(lobby);
}
