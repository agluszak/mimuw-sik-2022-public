use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::net::SocketAddr;

pub mod random;
pub mod serialize;

pub const MAX_UDP_LENGTH: usize = 65_535;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputMessage {
    PlaceBomb,
    PlaceBlock,
    Move { direction: Direction },
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMessage {
    Lobby {
        server_name: String,
        players_count: u8,
        size_x: u16,
        size_y: u16,
        game_length: u16,
        explosion_radius: u16,
        bomb_timer: u16,
        players: BTreeMap<PlayerId, Player>,
    },
    Game {
        server_name: String,
        size_x: u16,
        size_y: u16,
        game_length: u16,
        turn: u16,
        players: BTreeMap<PlayerId, Player>,
        player_positions: BTreeMap<PlayerId, Position>,
        blocks: HashSet<Position>,
        bombs: HashSet<Bomb>,
        explosions: HashSet<Position>,
        scores: BTreeMap<PlayerId, Score>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { name: String },
    PlaceBomb,
    PlaceBlock,
    Move { direction: Direction },
}

impl From<InputMessage> for ClientMessage {
    fn from(input_message: InputMessage) -> ClientMessage {
        match input_message {
            InputMessage::PlaceBomb => ClientMessage::PlaceBomb,
            InputMessage::PlaceBlock => ClientMessage::PlaceBlock,
            InputMessage::Move { direction } => ClientMessage::Move { direction },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bomb {
    pub position: Position,
    pub timer: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMessage {
    Hello {
        server_name: String,
        players_count: u8,
        size_x: u16,
        size_y: u16,
        game_length: u16,
        explosion_radius: u16,
        bomb_timer: u16,
    },
    AcceptedPlayer {
        id: PlayerId,
        player: Player,
    },
    GameStarted {
        players: BTreeMap<PlayerId, Player>,
    },
    Turn {
        turn: u16,
        events: Vec<Event>,
    },
    GameEnded {
        scores: BTreeMap<PlayerId, Score>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Score {
    pub deaths: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub socket_addr: SocketAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position(pub u16, pub u16);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Ord, PartialOrd,
)]
pub struct PlayerId(pub u8);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Ord, PartialOrd,
)]
pub struct BombId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    BombPlaced {
        id: BombId,
        position: Position,
    },
    BombExploded {
        id: BombId,
        killed: Vec<PlayerId>,
        blocks_destroyed: HashSet<Position>,
    },
    PlayerMoved {
        id: PlayerId,
        position: Position,
    },
    BlockPlaced {
        position: Position,
    },
}
