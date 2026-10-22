use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::*;

#[derive(Debug)]
pub enum Payload {
    // Serverbound
    Attack { entity_id: VarInt },
    UseItemOn {
        hand: VarInt,
        location: Position,
        face: VarInt,
        cursor_x: Float,
        cursor_y: Float,
        cursor_z: Float,
        inside_block: Bool,
        world_border: Bool,
        sequence: VarInt
    },

    // Clientbound
    AddEntity {
        id: VarInt,
        uuid: UUID,
        entity_type: VarInt,
        x: Double,
        y: Double,
        z: Double,
        velocity: LpVec3,
        pitch: UnsignedByte,
        yaw: UnsignedByte,
        head_yaw: UnsignedByte,
        data: VarInt
    }
}

impl Parse for Payload {
    fn parse(
        bytes: &mut &[u8],
        direction: Direction,
        id: VarInt
    ) -> Result<PhasePayload, DecodeError> {
        use Direction::{ Clientbound, Serverbound };
        use Payload::*;

        let id = id.0;

        let payload = match (direction, id) {
            (Serverbound, 1) => Attack { entity_id: VarInt::decode(bytes)? },
            (Serverbound, 66) => UseItemOn { 
                hand: VarInt::decode(bytes)?,
                location: Position::decode(bytes)?,
                face: VarInt::decode(bytes)?,
                cursor_x: Float::decode(bytes)?,
                cursor_y: Float::decode(bytes)?,
                cursor_z: Float::decode(bytes)?,
                inside_block: Bool::decode(bytes)?,
                world_border: Bool::decode(bytes)?,
                sequence: VarInt::decode(bytes)?
            },

            (Clientbound, 1) => {
                println!("Parsing add entity");

                AddEntity { 
                    id: VarInt::decode(bytes)?,
                    uuid: UUID::decode(bytes)?,
                    entity_type: VarInt::decode(bytes)?,
                    x: Double::decode(bytes)?,
                    y: Double::decode(bytes)?,
                    z: Double::decode(bytes)?,
                    velocity: LpVec3::decode(bytes)?,
                    pitch: UnsignedByte::decode(bytes)?,
                    yaw: UnsignedByte::decode(bytes)?,
                    head_yaw: UnsignedByte::decode(bytes)?,
                    data: VarInt::decode(bytes)?
                }
            },
            _ => Err(DecodeError::MalformedBytes)?
        };

        Ok(PhasePayload::Play(payload))
    }
}
