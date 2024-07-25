use std::{cmp::Ordering, collections::HashMap, iter::Map};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{region::Region, server::server_group::ServerGroup};

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
    pub server_count_map: HashMap<String, i16>,
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

impl DedicatedServerError {
    pub fn parsing_error_from_str(msg: &str) -> Self {
        Self::ParsingError(msg.into())
    }

    pub fn storage_error_from_str(msg: &str) -> Self {
        Self::StorageError(msg.into())
    }
}

impl DedicatedServer {
    fn get() -> Result<Vec<Self>, DedicatedServerError> {
        todo!()
    }

    pub fn get_server_count(&self, group: &ServerGroup) -> i16 {
        self.server_count_map.get(&group.name).unwrap_or(&0).clone()
    }

    fn increment_server_count(&mut self, group: &ServerGroup) -> Result<(), DedicatedServerError> {
        if !self.has_space_for(group) {
            return Err(DedicatedServerError::ParsingError(format!(
                "Dedicated Server ({:?}) has no space for server {:?} (try another dedicated server)",
                self.name, group.name
            )));
        }
        self.server_count_map
            .insert(group.name.clone(), self.get_server_count(group) + 1);
        self.available_ram -= group.ram as i16;
        self.available_cpu -= group.cpu as i16;
        Ok(())
    }

    fn decrement_server_count(&mut self, group: &ServerGroup) -> () {
        if let Some(&count) = self.server_count_map.get(&group.name) {
            self.server_count_map
                .insert(group.name.clone(), if count <= 1 { 0 } else { count - 1 });
            if count > 0 {
                // if old count wasn't zero (so available cpu or ram don't exceed max)
                self.available_ram += group.ram as i16;
                self.available_cpu += group.cpu as i16;
            }
        }
    }

    pub fn add_server(&mut self, group: &ServerGroup) -> Result<(), DedicatedServerError> {
        self.increment_server_count(group)
    }

    pub fn remove_server(&mut self, group: &ServerGroup) -> () {
        self.decrement_server_count(group);
    }

    pub fn has_space_for(&self, group: &ServerGroup) -> bool {
        self.available_ram >= (group.ram as i16) && self.available_cpu >= (group.cpu as i16)
    }
}
