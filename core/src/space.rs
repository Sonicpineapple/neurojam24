use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Direction {
    pub spatial: SpatialDirection,
    pub temporal: TemporalDirection,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpatialDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TemporalDirection {
    Forward,
    Backward,
}
