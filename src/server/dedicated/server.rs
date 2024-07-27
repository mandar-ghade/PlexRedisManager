use std::{cmp::Ordering, collections::HashMap, iter::Map};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    context_manager::ContextManager,
    region::Region,
    server::{minecraft::MinecraftServer, server_group::ServerGroup},
};

use super::instance::MCSInstance;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DedicatedServer {
    pub name: String,
    pub public_address: String,
    pub private_address: String,
    pub region: Region,
    #[serde(rename = "cpu")]
    pub available_cpu: i16,
    #[serde(rename = "ram")]
    pub available_ram: i16,
    #[serde(default = "ram_or_cpu_default")]
    pub max_cpu: i16,
    #[serde(default = "ram_or_cpu_default")]
    pub max_ram: i16,
    #[serde(skip)]
    pub server_instances: HashMap<String, Vec<MCSInstance>>,
    // pub waiting_to_start: Vec<MinecraftServer>,
    // ADD THIS so we can filter out for these
}

fn ram_or_cpu_default() -> i16 {
    0
}

#[derive(Error, Debug)]
pub enum DedicatedServerError {
    #[error("Dedicated Server Parsing Error: `{0}`")]
    ParsingError(String),
    #[error("Dedicated Server Storage Error: `{0}`")]
    StorageError(String),
    #[error("Dedicated Server Error: Bungee Not Found")]
    BungeeNotFoundError,
    #[error("Dedicated Server Error: Minecraft Server Not Running (took > 40 seconds): `{0}`")]
    MinecraftServerNotRunning(String),
    #[error("Dedicated Server Error: Duplicate instance of running: `{0}`")]
    DuplicateInstanceRunning(String),
    #[error("Dedicated Server Error: Minecraft Server Instance Not Found: `{0}`")]
    InstanceNotFound(String),
    #[error("Dedicated Server Error: Zero instances of ServerGroup online: `{0}`")]
    ZeroInstancesRunning(String),
}

impl Ord for DedicatedServer {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.available_ram, self.available_cpu).cmp(&(other.available_ram, other.available_cpu))
        // available ram most important, if self & other values equal compares available cpu
    }
}

impl PartialOrd for DedicatedServer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl DedicatedServer {
    fn get() -> Result<Vec<Self>, DedicatedServerError> {
        todo!()
    }

    pub fn get_instances(&self, group: &ServerGroup) -> Option<&Vec<MCSInstance>> {
        self.server_instances.get(&group.name)
    }

    pub fn get_server_count(&self, group: &ServerGroup) -> i16 {
        self.get_instances(group)
            .map(|vec| vec.len() as i16)
            .clone()
            .unwrap_or(0)
    }

    fn get_server_nums(&self, group: &ServerGroup) -> Vec<usize> {
        self.server_instances
            .get(&group.name)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|mcs| mcs.get_server_num())
            .collect()
    }

    pub fn launch_server(
        &mut self,
        group: &ServerGroup,
        server_num: usize,
        ctx: &mut ContextManager,
    ) -> Result<(), DedicatedServerError> {
        //! Launches server and waits every 5 seconds for the server to go online
        //! Times out after 40 seconds if it is not found in redis.
        assert_eq!(group.region, self.region);
        let mut server_name = group.name.clone();
        server_name.push_str(server_num.to_string().as_str());
        // now call shell script to run server
        let ticks = 0;
        loop {
            // todo: figure out how to increment tick
            todo!();
            if MinecraftServer::get(&server_name, &self.region, ctx).is_ok() {
                break;
            } else if ticks > 40 {
                return Err(DedicatedServerError::MinecraftServerNotRunning(server_name));
            }
        }
        Ok(())
    }

    pub fn add_server(
        &mut self,
        group: &ServerGroup,
        server_num: usize,
    ) -> Result<(), DedicatedServerError> {
        if !self.has_space_for(group) {
            return Err(DedicatedServerError::StorageError(
                format!("Dedicated Server ({:?}) has no space for server {:?} (try another dedicated server)",
                self.name, group.name
            )));
        } else if self.get_server_nums(group).contains(&server_num) {
            return Err(DedicatedServerError::DuplicateInstanceRunning(
                format!("Dedicated Server ({:?}) cannot run {:?} because this server instance is already running 
                (try another server number)",
                self.name, group.name
            )));
        }
        let group_name = group.name.clone();
        let server_name = format!("{}-{}", &group_name, &server_num);
        let instance: MCSInstance = MCSInstance::new(
            server_name,
            group.name.clone(),
            group.port_section + (server_num as u16),
            group.region.clone(),
            None,
        );
        if let Some(instances) = self.server_instances.get_mut(&group_name) {
            instances.push(instance);
        } else {
            let new_vec: Vec<MCSInstance> = Vec::from([instance]);
            self.server_instances.insert(group_name, new_vec);
        }
        self.available_ram -= group.ram as i16;
        self.available_cpu -= group.cpu as i16;
        Ok(())
    }

    pub fn remove_server(
        &mut self,
        group: &ServerGroup,
        server_num: usize,
    ) -> Result<(), DedicatedServerError> {
        let Some(vec) = self.server_instances.get_mut(&group.name) else {
            return Err(DedicatedServerError::ZeroInstancesRunning(
                format!("Dedicated Server ({:?}) cannot remove server because zero instances under {:?} were found",
                self.name, group.name
            )));
        };
        vec.sort_by(|a, b| a.get_server_num().cmp(&b.get_server_num()));

        if let Some(idx) = vec
            .iter()
            .position(|mcs| mcs.get_server_num() == server_num)
        {
            vec.swap_remove(idx);
        } else {
            let server_name: String = format!("{}-{}", group.name, server_num);
            return Err(DedicatedServerError::InstanceNotFound(format!(
                "Dedicated Server ({:?}) cannot remove {:?} because this instance was not found",
                self.name, server_name
            )));
        }
        self.available_ram += group.ram as i16;
        self.available_cpu += group.cpu as i16;
        Ok(())
    }

    pub fn has_space_for(&self, group: &ServerGroup) -> bool {
        self.available_ram >= (group.ram as i16) && self.available_cpu >= (group.cpu as i16)
    }
}
