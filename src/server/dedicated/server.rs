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
    pub server_count_map: HashMap<ServerGroup, i16>,
    // perhaps just make it by ServerGroup name so
    // conflicts will be reduced if changed
}

fn ram_or_cpu_default() -> i16 {
    0
}

#[derive(Error, Debug)]
pub enum DedicatedServerError {
    #[error("Dedicated Server Parsing Error: `{0}`")]
    ParsingError(String),
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

    pub fn get_server_count(&self, group: &ServerGroup) -> i16 {
        self.server_count_map.get(group).unwrap_or(&0).clone()
    }

    fn increment_server_count(&mut self, group: &ServerGroup) -> () {
        self.server_count_map.insert(
            group.clone(),
            self.server_count_map.get(group).unwrap_or(&(0 as i16)) + (1 as i16),
        );
        self.available_ram -= group.ram as i16;
        self.available_cpu -= group.cpu as i16;
    }

    fn decrement_server_count(&mut self, group: &ServerGroup) -> () {
        if let Some(&count) = self.server_count_map.get(group) {
            self.server_count_map
                .insert(group.clone(), if count <= 1 { 0 } else { count - 1 });
            if count > 0 {
                // if old count wasn't zero (so available cpu or ram don't exceed max)
                self.available_ram += group.ram as i16;
                self.available_cpu += group.cpu as i16;
            }
        }
    }

    pub fn add_server(&mut self, group: &ServerGroup) -> () {
        self.increment_server_count(group);
    }

    pub fn remove_server(&mut self, group: &ServerGroup) -> () {
        self.decrement_server_count(group);
    }

    pub fn has_space_for(&self, group: &ServerGroup) -> bool {
        self.available_ram >= (group.ram as i16) && self.available_cpu >= (group.cpu as i16)
    }
}
