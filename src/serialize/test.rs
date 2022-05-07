#![cfg(test)]

use crate::serialize::deserializer::from_bytes;
use crate::serialize::serializer::to_bytes;
use crate::{BombId, Event, PlayerId, Position, ServerMessage};

#[test]
fn server_message_events() {
    let sm = ServerMessage::Turn {
        turn: 44,
        events: vec![
            Event::PlayerMoved {
                id: PlayerId(3),
                position: Position(2, 4),
            },
            Event::PlayerMoved {
                id: PlayerId(4),
                position: Position(3, 5),
            },
            Event::BombPlaced {
                id: BombId(5),
                position: Position(5, 7),
            },
        ],
    };
    let bytes = to_bytes(&sm);
    println!("{:?}", bytes);
    let sm2 = from_bytes::<ServerMessage>(&to_bytes(&sm)).unwrap();
    assert_eq!(sm, sm2);
}
