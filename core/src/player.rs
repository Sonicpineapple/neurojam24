use serde::{Deserialize, Serialize};

use crate::Direction;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct PlayerAction {
    pub direction: Direction,
    pub action_type: ActionType,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionType {
    Move,
    Attack,
}
