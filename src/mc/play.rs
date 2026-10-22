use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::{ Double, String, UUID, UnsignedByte, VarInt };

#[derive(Debug)]
pub enum Payload {
    // Serverbound
    Attack { entity_id: VarInt },

    // Clientbound
    AddEntity {
        id: VarInt,
        uuid: UUID,
        entity_type: VarInt,
        x: Double,
        y: Double,
        z: Double,
        // velocity: LpVec3, // This is not correct
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
            _ => Err(DecodeError::MalformedBytes)?
        };

        Ok(PhasePayload::Play(payload))
    }
}
