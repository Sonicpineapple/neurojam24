use serde::{Deserialize, Serialize};

use crate::{DisplayData, GameResult, PlayerAction, PlayerStatus};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetBlob {
    Join,
    Assign(u8),
    Action(PlayerAction),
    Leave,
    Display(DisplayData),
    Result(GameResult),
    Stati([PlayerStatus; 2]),
    Start,
}
impl NetBlob {
    pub fn ser(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn deser(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
