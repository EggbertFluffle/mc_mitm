use crate::mc::{ DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::{ VarInt };

#[derive(Debug)]
pub enum Payload {
    // Serverbound
    FinishConfigurationAcknowledge(),
    
    // Clientbound
    FinishConfiguration(),
}

impl Parse for Payload {
    fn parse(
        _bytes: &mut &[u8],
        direction: Direction,
        id: VarInt
    ) -> Result<PhasePayload, DecodeError> {
        use Direction::{ Clientbound, Serverbound };
        use Payload::*;

        let id = id.0;

        let payload = match (direction, id) {
            (Serverbound, 3) => FinishConfigurationAcknowledge(),
            (Clientbound, 3) => FinishConfiguration(),
            _ => Err(DecodeError::MalformedBytes)?
        };

        Ok(PhasePayload::Configuration(payload))
    }
}
