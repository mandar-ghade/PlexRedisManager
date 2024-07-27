use crate::{
    context_manager::ContextManager,
    region::Region,
    server::minecraft::{MinecraftServer, ServerStatus},
};

/// Intermediate between ServerStatus cache
/// And DedicatedServer
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MCSInstance {
    name: String,
    group: String,
    server_num: usize,
    port: u16,
    region: Region,
    server: Option<MinecraftServer>,
}

impl MCSInstance {
    pub fn new(
        name: String,
        group: String,
        port: u16,
        region: Region,
        server: Option<MinecraftServer>,
    ) -> Self {
        Self {
            name: name.clone(),
            group,
            server_num: Self::calculate_server_num(&name),
            port,
            region,
            server,
        }
    }

    fn calculate_server_num(name: &String) -> usize {
        name.split_once('-')
            .unwrap_or((name.as_str(), "0"))
            .1
            .parse::<usize>()
            .ok()
            .unwrap_or(0)
    }

    pub fn get_server_num(&self) -> usize {
        self.server_num
    }

    pub fn get_status(&mut self, ctx: &mut ContextManager) -> ServerStatus {
        if let Some(sv) = self.server.as_mut() {
            return sv.update(ctx);
        }
        match MinecraftServer::get(&self.name, &self.region, ctx) {
            Ok(mut server) => {
                let status = server.update(ctx);
                self.server = Some(server);
                status
            }
            Err(_) => ServerStatus::DOES_NOT_EXIST,
        }
    }

    pub fn get_mcs(&mut self, ctx: &mut ContextManager) -> Option<&mut MinecraftServer> {
        self.server.as_mut()
    }
}
