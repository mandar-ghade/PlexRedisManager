use serde::{Deserialize, Serialize};

pub mod collection;
pub mod instance;
pub mod server;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct System {
    pub system: SystemName,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SystemName {
    Linux,
    Mac,
    Windows,
}
