use crate::mc::{self, types::VarInt};

use mc::types::VarInt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handshake {
    length: VarInt,
    id: VarInt
}

impl mc::Decode for Handshake {
    fn decode(bytes: &mut &[u8]) -> Result<Self, String> {
        Ok(Self {
            length: VarInt::decode(bytes)?,
            id: VarInt::decode(bytes)?,
        })
    }
}
