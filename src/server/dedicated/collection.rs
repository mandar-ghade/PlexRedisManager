use std::usize;

use serde::{Deserialize, Serialize};

use crate::server::{minecraft::MinecraftServer, server_group::ServerGroup};

use super::server::DedicatedServer;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DedicatedServers {
    pub servers: Vec<DedicatedServer>,
}

impl DedicatedServers {
    pub fn get_best_dedicated_server(
        &mut self,
        group: &ServerGroup,
    ) -> Option<&mut DedicatedServer> {
        //! Gets server with highest resources which can fulfill a servergroup's resource requirement.
        //! Gets best server with highest resources and lowest server count for the specific group.
        self.sort_servers();

        let mut best_server: Option<&mut DedicatedServer> = None;
        for ds in self.servers.iter_mut() {
            if ds.region != group.region || !ds.has_space_for(group) {
                continue;
            }
            if let Some(best) = best_server.as_ref() {
                // it isn't the best if it doesn't have a lower server count
                if best.get_server_count(group) < ds.get_server_count(group) {
                    continue;
                }
            }
            best_server = Some(ds);
        }
        best_server
    }

    pub fn get_running_servers(&mut self) -> Vec<MinecraftServer> {
        //! Get running minecraft servers across all nodes
        todo!()
    }

    fn get_highest_server_num(&self, group: &ServerGroup) -> usize {
        self.servers
            .iter()
            .map(|ds| {
                ds.server_instances
                    .get(&group.name)
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|mcs| mcs.get_server_num())
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0)
    }

    pub fn get_next_server_num(&self, group: &ServerGroup) -> usize {
        self.get_highest_server_num(group) + 1
    }

    pub fn get_next(&mut self) -> Option<DedicatedServer> {
        self.servers.clone().into_iter().next()
    }

    fn sort_servers(&mut self) -> () {
        //! Sorts DedicatedServers by highest resource first (ram more important, then cpu)
        self.servers.sort();
    }
}
