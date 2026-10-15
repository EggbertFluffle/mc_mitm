use crate::mc::{ Decode, DecodeError };

pub type VarInt = i32;

impl Decode for VarInt {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        let mut value: i32 = 0;

        for i in 0..5 {
            let b: &u8 = match bytes.get(i) {
                None => { return Err(DecodeError::PrematureByteEnd); },
                Some(b) => b
            };

            value |= ((b & 0x7F) as i32) << (i * 7);

            if b & 0x80 == 0 { 
                *bytes = &bytes[i + 1..];
                return Ok(value);
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
            Ok(*b != 0)
        } else {
            Err(DecodeError::PrematureByteEnd)
        }
    }
}

pub type Byte = i8;

impl Decode for Byte {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0) {
            *bytes = &bytes[1..];
            Ok(*b as i8) // TODO: Double check this reinterprets the byte
        } else {
            Err(DecodeError::PrematureByteEnd)
        }
    }
}

pub type UnsignedByte = u8;

impl Decode for UnsignedByte {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        if let Some(b) = bytes.get(0) {
            *bytes = &bytes[1..];
            Ok(*b)
        } else {
            Err(DecodeError::PrematureByteEnd)
        }
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

            Err(DecodeError::PrematureByteEnd)
        } else {
            Err(DecodeError::PrematureByteEnd)
        }
    }
}
