use crate::mc;

pub type VarInt = i32;

impl mc::Decode for VarInt {
    fn decode(bytes: &mut &[u8]) -> Result<Self, String> {
        let mut value: i32 = 0;

        for i in 0..5 {
            let b: u8 = bytes[i];

            value |= ((b & 0x7F) as i32) << (i * 7);

            if b & 0x80 == 0 { 
                *bytes = &bytes[i + 1..];
                return Ok(value);
            }
        }

        Err("VarInt too big".to_string())
    }
}
