use crate::mc::{self, types::VarInt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handshake {
    length: VarInt,
    id: VarInt
}

impl mc::Decode for Handshake {
    fn decode(bytes: &mut &[u8]) -> Result<Self, mc::DecodeError> {
        Ok(Self {
            length: VarInt::decode(bytes)?,
            id: VarInt::decode(bytes)?,
        })
    }
}
