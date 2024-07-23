use crate::{
    context_manager::ContextManager, error::parsing_error::ServerGroupParsingError,
    game::utils::GENERIC_TO_SERVER_GROUP,
};

use super::server_group::ServerGroup;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum GenericServer {
    Lobby,
    ClansHub,
    BetaHub,
}

impl GenericServer {
    pub fn to_server_group(
        &self,
        ctx: &mut ContextManager,
    ) -> Result<Option<ServerGroup>, ServerGroupParsingError> {
        //! Converts GenericServer to ServerGroup. Loads from cache if exists.
        GENERIC_TO_SERVER_GROUP
            .get(self)
            .map(|sg| sg.clone())
            .map_or(Ok(None), |mut group| {
                group.eliminate_port_collisions(ctx)?;
                group.load_existing_cache(ctx);
                Ok(Some(group))
            })
    }
}
