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
        match bytes.get(0) {
            None => Err(DecodeError::PrematureByteEnd),
            Some(b) => {
                *bytes = &bytes[1..];
                Ok(*b != 0)
            }
        }
    }
}

pub type Byte = i8;

impl Decode for Byte {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        match bytes.get(0) {
            None => Err(DecodeError::PrematureByteEnd),
            Some(b) => {
                *bytes = &bytes[1..];
                Ok(*b as i8) // TODO: Double check this reinterprets the byte
            }
        }
    }
}

pub type UnsignedByte = u8;

impl Decode for UnsignedByte {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> {
        match bytes.get(0) {
            None => Err(DecodeError::PrematureByteEnd),
            Some(b) => {
                *bytes = &bytes[1..];
                Ok(*b)
            }
        }
    }
}
