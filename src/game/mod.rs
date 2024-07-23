use crate::context_manager::ContextManager;
use crate::error::parsing_error::ServerGroupParsingError;
use crate::game::options::GameOptions;
use crate::game::r#type::GameType;
pub mod booster_group;
use std::str::FromStr;
pub mod options;
pub mod r#type;
pub mod utils;

#[derive(Debug)]
pub struct Game {
    #[allow(dead_code)]
    pub name: GameType,
    pub options: GameOptions,
}

impl Game {
    pub fn from_game_type(
        game: GameType,
        ctx: &mut ContextManager,
    ) -> Result<Self, ServerGroupParsingError> {
        Ok(Self {
            name: game,
            options: GameOptions::from_game_type(game, ctx)?,
        })
    }

    pub fn from_str(game: &str, ctx: &mut ContextManager) -> Result<Self, ServerGroupParsingError> {
        let game_name: GameType = GameType::from_str(game)
            .map_err(|err| ServerGroupParsingError::new(format!("Game not found: {:?}", err)))?;
        Ok(Self {
            name: game_name,
            options: GameOptions::from_game_type(game_name, ctx)?,
        })
    }
}
