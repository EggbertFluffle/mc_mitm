use crate::mc::{ Decode, DecodeError };

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

pub type UnsignedInt = u32;

impl Decode for UnsignedInt {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..4) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[4..];
                return Ok(u32::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

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

pub type UnsignedLong = u64;

impl Decode for UnsignedLong {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..8) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[8..];
                return Ok(u64::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

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

pub type Double = f64;

impl Decode for Double {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0..8) {
            if let Ok(b) = b.try_into() {
                *bytes = &bytes[8..];
                return Ok(f64::from_be_bytes(b));
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct LpVec3 {
    x: Double,
    y: Double,
    z: Double
}

impl Decode for LpVec3 {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        const MAX_QUANTIZED_VALUE: f64 = 32766.0;
        const CONTINUATION_FLAG: u8 = 0x04;
        const SCALE_BITS: u64 = 0x03;

        let byte1 = UnsignedByte::decode(bytes)?;
        if byte1 == 0 {
            return Ok(LpVec3 { x: 0.0, y: 0.0, z: 0.0 });
        }

        let byte2 = UnsignedByte::decode(bytes)?;
        let bytes3_to6 = UnsignedInt::decode(bytes)?;

        let packed: u64 = ((bytes3_to6 as u64) << 16) | ((byte2 as u64) << 8) | (byte1 as u64);

        let mut scale_factor: u64 = (byte1 as u64) & SCALE_BITS;
        if byte1 & CONTINUATION_FLAG != 0 {
            let VarInt(b) = VarInt::decode(bytes)?;
            scale_factor |= (b as u64) << 2;
        }
        let scale = scale_factor as f64;

        let unpack = |value: u64| ((value & 32767) as f64).min(MAX_QUANTIZED_VALUE) * 2.0 / MAX_QUANTIZED_VALUE - 1.0;

        Ok(LpVec3 {
            x: unpack(packed >> 3) * scale,
            y: unpack(packed >> 18) * scale,
            z: unpack(packed >> 33) * scale,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    x: i32,
    y: i32,
    z: i32
}

impl Decode for Position {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        let val = Long::decode(bytes)?;

        Ok(Position { 
            x: (val >> 38) as i32,
            y: ((val << 52) >> 52) as i32,
            z: ((val << 26) >> 38) as i32
        })
    }
}

pub type PrefixedOptional<T> = Option<T>;

impl<T: Decode> Decode for PrefixedOptional<T> {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0) {
            return match *b {
                0 => Ok(None),
                _ => {
                    *bytes = &bytes[1..];
                    Ok(Some(T::decode(bytes)?))
                }
            }
        }

        Err(DecodeError::PrematureByteEnd)
    }
}

pub type PrefixedArray<T> = Vec<T>;

impl<T: Decode> Decode for PrefixedArray<T> {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        let VarInt(length) = VarInt::decode(bytes)?;

        let mut out: Vec<T> = Vec::new();
        out.reserve_exact(length as usize);
        for _ in 0..length as usize {
            out.push(T::decode(bytes)?);
        }
        Ok(out)
    }
}

#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R)
}

impl<L: Decode, R: Decode> Decode for Either<L, R> {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(if Bool::decode(bytes)? {
            Either::Left(L::decode(bytes)?)
        } else {
            Either::Right(R::decode(bytes)?)
        })
    }
}

// Handles the parsing of compound, tuple types
macro_rules! impl_decode_tuple {
    ($($T:ident),+) => {
        impl<$($T: Decode),+> Decode for ($($T,)+) {
            fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
                Ok(($($T::decode(bytes)?,)+))
            }
        }
    };
}

impl_decode_tuple!(A);
impl_decode_tuple!(A, B);
impl_decode_tuple!(A, B, C);
impl_decode_tuple!(A, B, C, D);
