use crate::{Direction, PlayerAction, PlayerStatus, SpatialDirection, TemporalDirection};

#[derive(Debug, Default, Copy, Clone)]
// pub enum TileState {
//     #[default]
//     Empty,
//     Player(u8),
//     Hazard,
// }
// impl TileState {
//     pub fn char(self) -> char {
//         match self {
//             TileState::Empty => 'E',
//             TileState::Player(n) => n.to_string().chars().next().expect("lmao"),
//             TileState::Hazard => 'X',
//         }
//     }
// }

pub struct TileState {
    player: Option<u8>,
    player_status: Option<PlayerStatus>,
    hazard: bool,
}
impl TileState {
    pub fn is_empty(&self) -> bool {
        self.player.is_none() && self.hazard == false
    }
    pub fn is_movable(&self) -> bool {
        self.player.is_none()
    }
    pub fn is_hazard(&self) -> bool {
        self.hazard
    }
    pub fn status(&self) -> Option<PlayerStatus> {
        self.player_status
    }

    pub fn set_status(&mut self, status: Option<PlayerStatus>) {
        self.player_status = status;
    }

    pub fn char(&self) -> char {
        if self.is_empty() {
            'E'
        } else if let Some(player) = self.player {
            player.to_string().chars().next().expect("lmao")
        } else {
            'X'
        }
    }

    fn player(player_id: u8) -> Self {
        TileState {
            player: Some(player_id),
            player_status: None,
            hazard: false,
        }
    }
}

pub const SIZE: usize = 7;
pub const LENGTH: usize = 5;

#[derive(Debug, Clone)]
pub struct BoardState {
    tiles: [[TileState; SIZE]; SIZE],
}
impl BoardState {
    pub fn empty() -> Self {
        Self {
            tiles: [[TileState::default(); SIZE]; SIZE],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Result<TileState, ()> {
        if x < SIZE && y < SIZE {
            return Ok(self.tiles[y][x]);
        }
        Err(())
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> Result<&mut TileState, ()> {
        if x < SIZE && y < SIZE {
            return Ok(&mut self.tiles[y][x]);
        }
        Err(())
    }
    pub fn set(&mut self, x: usize, y: usize, state: TileState) -> bool {
        if x < SIZE && y < SIZE {
            self.tiles[y][x] = state;
            return true;
        }
        false
    }
    pub fn tiles(&self) -> &[[TileState; SIZE]; SIZE] {
        &self.tiles
    }
}
impl std::fmt::Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.tiles {
            for tile in row {
                write!(f, "{}", tile.char())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    states: [BoardState; LENGTH],
}
impl Board {
    pub const SPAWNS: [Stamp; 2] = [
        Stamp {
            x: SIZE / 2,
            y: 1,
            t: 0,
        },
        Stamp {
            x: SIZE / 2,
            y: SIZE - 2,
            t: 0,
        },
    ];
    pub fn initial() -> Self {
        let mut board = Self {
            states: core::array::from_fn(|_| BoardState::empty()),
        };
        for i in 0..2 {
            board.set(Self::SPAWNS[i], TileState::player(i as u8));
        }
        board
    }

    pub fn get(&self, stamp: Stamp) -> Result<TileState, ()> {
        if stamp.t < LENGTH {
            return self.states[stamp.t].get(stamp.x, stamp.y);
        }
        Err(())
    }
    fn get_mut(&mut self, stamp: Stamp) -> Result<&mut TileState, ()> {
        if stamp.t < LENGTH {
            return self.states[stamp.t].get_mut(stamp.x, stamp.y);
        }
        Err(())
    }

    fn set(&mut self, stamp: Stamp, state: TileState) -> bool {
        if stamp.t < LENGTH {
            return self.states[stamp.t].set(stamp.x, stamp.y, state);
        }
        false
    }
    pub fn set_status(&mut self, stamp: Stamp, status: Option<PlayerStatus>) {
        if stamp.t < LENGTH {
            if let Ok(tile) = self.states[stamp.t].get_mut(stamp.x, stamp.y) {
                tile.set_status(status);
            };
        }
    }

    pub fn states(&self) -> &[BoardState; LENGTH] {
        &self.states
    }

    pub fn process_impact(&mut self, impact: Impact) {
        let Impact {
            player: (player_stamp, player_id),
            attack,
        } = impact;
        let player_target_tile = self
            .get_mut(player_stamp)
            .expect("Should be validated already");
        player_target_tile.player = Some(player_id);

        if let Some(attack_stamp) = attack {
            let attack_target_tile = self
                .get_mut(attack_stamp)
                .expect("Should be validated already");
            attack_target_tile.hazard = true;
        }
    }

    pub fn calculate_action(
        &self,
        player_id: usize,
        action: PlayerAction,
        source: Stamp,
    ) -> Result<(Stamp, Impact), Error> {
        let mut target = source.clone();
        target = target + action.direction;

        let mut player = None;
        let mut attack = None;
        if let Ok(state) = self.get(target) {
            match action.action_type {
                crate::ActionType::Move => {
                    if state.is_movable() {
                        player = Some((target, player_id as u8));
                    } else {
                        dbg!(player_id);
                        dbg!(source);
                        return Err(Error::InvalidMove("Target is occupied"));
                    }
                }
                crate::ActionType::Attack => {
                    let player_dest = source + action.direction.temporal;
                    if !self
                        .get(player_dest)
                        .expect("They're already here come on")
                        .is_movable()
                    {
                        return Err(Error::InvalidMove("Target is occupied (stationary)"));
                    }
                    player = Some((player_dest, player_id as u8));
                    attack = Some(target);
                }
            }
        }
        if let Some(player) = player {
            Ok((source, Impact { player, attack }))
        } else {
            Err(Error::InvalidMove("Illegal state"))
        }
    }
}
impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for state in &self.states {
            writeln!(f, "{}", state)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub x: usize,
    pub y: usize,
    pub t: usize,
}
impl std::ops::Add<SpatialDirection> for Stamp {
    type Output = Self;

    fn add(self, rhs: SpatialDirection) -> Self::Output {
        let mut out = self.clone();
        match rhs {
            SpatialDirection::Left => out.x -= 1,
            SpatialDirection::Right => out.x += 1,
            SpatialDirection::Up => out.y -= 1,
            SpatialDirection::Down => out.y += 1,
        };
        out
    }
}
impl std::ops::Add<TemporalDirection> for Stamp {
    type Output = Self;

    fn add(self, rhs: TemporalDirection) -> Self::Output {
        let mut out = self.clone();
        match rhs {
            TemporalDirection::Forward => out.t += 1,
            TemporalDirection::Backward => out.t -= 1,
        };
        out
    }
}
impl std::ops::Add<Direction> for Stamp {
    type Output = Self;

    fn add(self, rhs: Direction) -> Self::Output {
        self + rhs.spatial + rhs.temporal
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidMove(&'static str),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Impact {
    pub player: (Stamp, u8),
    pub attack: Option<Stamp>,
}
