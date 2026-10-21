use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::{ String, VarInt, GameProfile, UUID };

#[derive(Debug)]
pub enum Payload {
    // Serverbound
    Attack { entity_id: VarInt }
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
