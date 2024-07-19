use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::error::parsing_error::ServerGroupParsingError;

#[derive(Clone, Debug, Display, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum Region {
    US,
    EU,
    ALL,
}

impl Default for Region {
    fn default() -> Self {
        Region::US
    }
}

impl TryFrom<String> for Region {
    type Error = ServerGroupParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "US" | "" => Ok(Region::US),
            "EU" => Ok(Region::EU),
            "ALL" => Ok(Region::ALL),
            _ => Err(ServerGroupParsingError::new(
                "Region could not be parsed.".into(),
            )),
        }
    }
}

impl From<Region> for String {
    fn from(region: Region) -> Self {
        match region {
            Region::US => "US".into(),
            Region::EU => "EU".into(),
            Region::ALL => "ALL".into(),
        }
    }
}
