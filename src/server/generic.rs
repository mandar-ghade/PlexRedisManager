use std::borrow::BorrowMut;

use crate::{error::parsing_error::ServerGroupParsingError, game::utils::GENERIC_TO_SERVER_GROUP};

use super::server_group::ServerGroup;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum GenericServer {
    Lobby,
    ClansHub,
    BetaHub,
}

impl GenericServer {
    pub fn to_server_group(&self) -> Result<Option<ServerGroup>, ServerGroupParsingError> {
        //! Converts GenericServer to ServerGroup. Loads from cache if exists.
        if let Some(mut group) = GENERIC_TO_SERVER_GROUP
            .get(self)
            .map(|sg| sg.clone().to_owned())
        {
            group.minimize_port_collisions()?;
            group.load_existing_cache();
            Ok(Some(group))
        } else {
            Ok(None)
        }
    }
}
