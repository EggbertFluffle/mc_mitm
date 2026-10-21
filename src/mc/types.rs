use crate::mc::{ Decode, DecodeError };

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarInt(pub i32);

impl Decode for VarInt {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        let mut value: i32 = 0;

        for i in 0..5 {
            let b: &u8 = match bytes.get(i) {
                None => { Err(DecodeError::PrematureByteEnd)? },
                Some(b) => b
            };

            value |= ((b & 0x7F) as i32) << (i * 7);

            if b & 0x80 == 0 { 
                *bytes = &bytes[i + 1..];
                return Ok(VarInt(value));
            }
        }

        Err(DecodeError::MalformedBytes)
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VarLong(pub i64);

impl Decode for VarLong {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        let mut value: i64 = 0;

        for i in 0..10 {
            let b: &u8 = match bytes.get(i) {
                None => { Err(DecodeError::PrematureByteEnd)? },
                Some(b) => b
            };

            value |= ((b & 0x7F) as i64) << (i * 7);

            if b & 0x80 == 0 { 
                *bytes = &bytes[i + 1..];
                return Ok(VarLong(value));
            }
        }

        Err(DecodeError::MalformedBytes)
    }
}

#[allow(unused)]
pub type Bool = bool;

impl Decode for Bool {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0) {
            *bytes = &bytes[1..];
            return Ok(*b != 0)
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type Byte = i8;

impl Decode for Byte {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0) {
            *bytes = &bytes[1..];
            return Ok(*b as i8)
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type UnsignedByte = u8;

impl Decode for UnsignedByte {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0) {
            *bytes = &bytes[1..];
            return Ok(*b)
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type Short = i16;

impl Decode for Short {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..2) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[2..];
                return Ok(i16::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type UnsignedShort = u16;

impl Decode for UnsignedShort {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..2) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[2..];
                return Ok(u16::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type Int = i32;

impl Decode for Int {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..4) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[4..];
                return Ok(i32::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type Long = i64;

impl Decode for Long {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..8) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[8..];
                return Ok(i64::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type Float = f32;

impl Decode for Float {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..4) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[4..];
                return Ok(f32::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type Double = f64;

impl Decode for Double {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..4) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[4..];
                return Ok(f64::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
pub type String = std::string::String;

impl Decode for String {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        let VarInt(length) = VarInt::decode(bytes)?;

        if let Some(b) = bytes.get(0..length as usize) {
            let s = String::from_utf8(b.to_vec())
                .map_err(|_| { DecodeError::MalformedBytes })?;
            *bytes = &bytes[s.len()..];
            return Ok(s)
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UUID(pub u128);

impl Decode for UUID {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..16) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[16..];
                return Ok(UUID(u128::from_be_bytes(b)));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
#[derive(Debug)]
pub enum PrefixedOptional<T: Decode> {
    PrefixedNone,
    PrefixedSome(T)
}

impl<T: Decode> Decode for PrefixedOptional<T> {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        use PrefixedOptional::*;

        if let Some(b) = bytes.get(0) {
            return match *b {
                0 => Ok(PrefixedNone),
                _ => {
                    *bytes = &bytes[1..];
                    Ok(PrefixedSome(T::decode(bytes)?))
                }
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GameProfile {
    uuid: UUID,
    username: String,
    // properties
}

impl Decode for GameProfile {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        return Ok(GameProfile {
            uuid: UUID::decode(bytes)?,
            username: String::decode(bytes)?,
        });
    }
}
