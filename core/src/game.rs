use serde::{Deserialize, Serialize};

use crate::{
    Board, DisplayData, Error, Impact, PlayerAction, Stamp, TileDisplayData, LENGTH, SIZE,
};

pub struct GameStatus {
    pub board: Board,
    pub player_actions: [Vec<(Stamp, PlayerAction, Impact)>; 2],
    pub player_stati: [PlayerStatus; 2],
    pub player_locations: [Stamp; 2],
}
impl GameStatus {
    pub fn initial() -> Self {
        Self {
            board: Board::initial(),
            player_actions: [const { Vec::new() }; 2],
            player_stati: [PlayerStatus::new(); 2],
            player_locations: Board::SPAWNS,
        }
    }

    pub fn push_actions(
        &mut self,
        actions: [PlayerAction; 2],
    ) -> Result<Option<GameResult>, Error> {
        let mut results = actions.iter().enumerate().map(|(player_id, &action)| {
            self.board
                .calculate_action(player_id, action, self.player_locations[player_id])
        });
        let results = [results.next().expect("oof")?, results.next().expect("oof")?];
        if results[0].1.player.0 == results[1].1.player.0 {
            // filling the same tile
            return Err(Error::InvalidMove("Bonked lmao"));
        }
        for (i, &e) in results.iter().enumerate() {
            self.player_actions[i].push((e.0, actions[i], e.1));
            self.player_locations[i] = e.1.player.0
        }
        self.evaluate_actions()
    }

    pub fn evaluate_actions(&mut self) -> Result<Option<GameResult>, Error> {
        // actions are assumed to be legal, so they can be evaluated per player
        let Self {
            board,
            player_actions,
            player_stati,
            player_locations,
        } = self;

        // Follow each timeline to find effects on each tile
        for player_actions in player_actions.iter() {
            for (_source, _action, impact) in player_actions {
                board.process_impact(*impact);
            }
        }

        // Follow each timeline again to process effects on each player
        *player_stati = [PlayerStatus::new(); 2];
        let mut check_damage = |player_id: usize, stamp, tick: bool| {
            if board.get(stamp).expect("Presumed valid").is_hazard() {
                player_stati[player_id].damage();
            }
            board.set_status(stamp, Some(player_stati[player_id].clone()));
            if tick {
                player_stati[player_id].tick();
            }
        };
        for (player_id, player_actions) in player_actions.iter().enumerate() {
            for (source, _action, _impact) in player_actions {
                check_damage(player_id, *source, true);
            }
        }
        for (player_id, stamp) in player_locations.iter().enumerate() {
            check_damage(player_id, *stamp, false);
        }
        // for status in player_stati.iter_mut() {
        //     status.tick();
        // }
        Ok(match player_stati.map(|s| s.health) {
            [0, 0] => Some(GameResult::Draw),
            [0, _] => Some(GameResult::Win(1)),
            [_, 0] => Some(GameResult::Win(0)),
            _ => None,
        })
    }

    pub fn display(&self) -> DisplayData {
        let mut data = std::array::from_fn(|_| {
            std::array::from_fn(|_| std::array::from_fn(|_| TileDisplayData::new()))
        });
        for (player_id, player_actions) in self.player_actions.iter().enumerate() {
            for (source, action, impact) in player_actions {
                let current_tile = &mut data[source.t][source.y][source.x];
                current_tile
                    .set_player(
                        player_id as u8,
                        false,
                        self.board.get(*source).expect("what").status().unwrap(),
                    )
                    .unwrap();
                current_tile.set_outgoing(*action).unwrap();

                let (target, player_id) = impact.player;
                let moved_tile = &mut data[target.t][target.y][target.x];
                moved_tile.add_incoming_move(player_id, *action);

                if let Some(attack) = impact.attack {
                    let attacked_tile = &mut data[attack.t][attack.y][attack.x];
                    attacked_tile.add_incoming_attack(player_id, *action);
                }
            }
        }
        for (player_id, stamp) in self.player_locations.iter().enumerate() {
            data[stamp.t][stamp.y][stamp.x]
                .set_player(player_id as u8, true, self.player_stati[player_id])
                .unwrap();
        }
        data
    }
}

pub enum TurnStatus {
    Active(u8),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct PlayerStatus {
    pub health: u8,
    pub iframes: u8,
    pub time: usize,
}
impl PlayerStatus {
    pub const fn new() -> Self {
        Self {
            health: 3,
            iframes: 0,
            time: 0,
        }
    }
    pub fn is_vulnerable(&self) -> bool {
        self.iframes == 0
    }
    pub fn is_defeated(&self) -> bool {
        self.health == 0
    }

    pub fn damage(&mut self) {
        if self.is_vulnerable() {
            self.health -= 1;
            self.iframes = 3;
        }
    }

    pub fn tick(&mut self) {
        if self.iframes > 0 {
            self.iframes -= 1
        }
        self.time += 1
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameResult {
    Win(u8),
    Draw,
}
