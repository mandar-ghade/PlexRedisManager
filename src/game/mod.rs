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

impl TryFrom<GameType> for Game {
    type Error = ServerGroupParsingError;
    fn try_from(game: GameType) -> Result<Self, Self::Error> {
        Ok(Self {
            name: game,
            options: GameOptions::try_from(game)?,
        })
    }
}

impl TryFrom<&str> for Game {
    type Error = ServerGroupParsingError;
    fn try_from(game: &str) -> Result<Self, Self::Error> {
        let game_name = GameType::from_str(game)
            .map_err(|err| ServerGroupParsingError::new(format!("Game not found: {:?}", err)))?;
        Ok(Self {
            name: game_name,
            options: GameOptions::try_from(game_name)?,
        })
    }
}
