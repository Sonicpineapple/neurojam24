use serde::{Deserialize, Serialize};

use crate::{PlayerAction, LENGTH, SIZE};

pub type DisplayData = [[[TileDisplayData; SIZE]; SIZE]; LENGTH];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TileDisplayData {
    player: Option<(u8, bool)>, // Which player is here and whether this is their "current" position
    hazard: bool,               // Whether this tile will cause damage
    outgoing: Option<PlayerAction>, // What action is happening in this cell
    incoming: Vec<(u8, PlayerAction, bool)>, // What actions are about to affect this cell, who did them, and whether they are movement
}
impl TileDisplayData {
    pub const fn new() -> Self {
        Self {
            player: None,
            hazard: false,
            outgoing: None,
            incoming: Vec::new(),
        }
    }

    pub fn set_player(&mut self, player_id: u8, active: bool) -> Result<(), ()> {
        if self.player.is_some() {
            return Err(());
        }
        self.player = Some((player_id, active));
        Ok(())
    }

    pub fn set_outgoing(&mut self, action: PlayerAction) -> Result<(), ()> {
        if self.outgoing.is_some() {
            return Err(());
        }
        self.outgoing = Some(action);
        Ok(())
    }

    pub fn add_incoming_move(&mut self, player_id: u8, action: PlayerAction) {
        self.add_incoming(player_id, action, false);
    }
    pub fn add_incoming_attack(&mut self, player_id: u8, action: PlayerAction) {
        self.add_incoming(player_id, action, true);
        self.hazard = true;
    }
    pub fn add_incoming(&mut self, player_id: u8, action: PlayerAction, attack: bool) {
        self.incoming.push((player_id, action, attack));
    }

    pub fn player(&self) -> Option<(u8, bool)> {
        self.player
    }
    pub fn outgoing(&self) -> Option<PlayerAction> {
        self.outgoing
    }
    pub fn incoming(&self) -> &Vec<(u8, PlayerAction, bool)> {
        &self.incoming
    }

    pub fn is_empty(&self) -> bool {
        self.player.is_none()
    }
    pub fn is_attacked(&self) -> bool {
        self.hazard
    }
    pub fn incoming_attacks(&self) -> Vec<(u8, PlayerAction)> {
        self.incoming
            .iter()
            .filter(|(_, _, attack)| *attack)
            .map(|(id, action, _)| (*id, *action))
            .collect()
    }
}
