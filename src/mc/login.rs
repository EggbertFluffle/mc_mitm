use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::{ String, VarInt, GameProfile, UUID };

#[derive(Debug)]
pub enum Payload {
    // Serverbound packets
    LoginStart { name: String, uuid: UUID },
    EncryptionResponse(), // Cooked
    LoginAcknowledged(),

    // Clientbound packets
    Disconnect { reason: String }, // Actually a JSON Text Component, not String
    EncryptionRequest(), // Cooked
    LoginFinished { profile: GameProfile, session_id: UUID },
    SetCompression { threshold: VarInt }
}

impl Parse for Payload {
    fn parse(
        bytes: &mut &[u8],
        direction: Direction,
        id: VarInt
    ) -> Result<Vec<PhasePayload>, DecodeError> {
        use Direction::{ Clientbound, Serverbound };
        use Payload::*;

        let id = id.0;

        let payload = match (direction, id) {
            (Serverbound, 0) => LoginStart { 
                name: String::decode(bytes)?,
                uuid: UUID::decode(bytes)?
            },
            (Serverbound, 1) => EncryptionRequest(),
            (Serverbound, 3) => LoginAcknowledged(),

            (Clientbound, 0) => Disconnect { reason: String::decode(bytes)? },
            (Clientbound, 1) => EncryptionRequest(),
            (Clientbound, 2) => LoginFinished { 
                profile: GameProfile::decode(bytes)?,
                session_id: UUID::decode(bytes)?
            },
            (Clientbound, 3) => SetCompression { threshold: VarInt::decode(bytes)? },
            _ => Err(DecodeError::MalformedBytes)?
        };

        Ok(vec![PhasePayload::Login(payload)])
    }
}
