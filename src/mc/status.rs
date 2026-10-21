use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::{ String, VarInt, Long };

#[derive(Debug)]
pub enum Payload {
    PingResponse { timestamp: Long },
    StatusRequest,
    PingRequest { timestamp: Long },
    StatusResponse { json: String }
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
            (Clientbound, 0) => StatusResponse { json: String::decode(bytes)? },
            (Clientbound, 1) => PingResponse { timestamp: Long::decode(bytes)? },
            (Serverbound, 0) => StatusRequest,
            (Serverbound, 1) => PingRequest { timestamp: Long::decode(bytes)? },
            _ => Err(DecodeError::MalformedBytes)?
        };

        Ok(PhasePayload::Status(payload))
    }
}
