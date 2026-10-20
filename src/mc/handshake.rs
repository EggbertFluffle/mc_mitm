use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::{ VarInt, UnsignedShort };

#[derive(Debug)]
pub enum Payload {
    Intention{
        protocol_version: VarInt,
        address: String,
        port: UnsignedShort,
        intent: VarInt
    }
}

impl Parse for Payload {
    fn parse(
        bytes: &mut &[u8],
        direction: Direction,
        id: VarInt
    ) -> Result<PhasePayload, DecodeError> {
        use Direction::Serverbound;

        let id = id.0;

        let payload = match (direction, id) {
            (Serverbound, 0) => Payload::Intention {
                protocol_version: VarInt::decode(bytes)?,
                address: String::decode(bytes)?,
                port: UnsignedShort::decode(bytes)?,
                intent: VarInt::decode(bytes)?
            },
            _ => Err(DecodeError::MalformedBytes)?
        };

        Ok(PhasePayload::Handshake(payload))
    }
}
