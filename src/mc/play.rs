use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload, types::VarInt };

pub enum Payload {
    Attack,
    OpenContainer
}
