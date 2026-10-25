use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::*;

#[derive(Debug)]
pub enum Payload {
    // Serverbound
    Attack { entity_id: VarInt },
    ClientTickEnd(),
    KeepServerAlive { id: Long },
    PlayerPosition {
        x: Double,
        y: Double,
        z: Double,
        flags: Byte
    },
    PlayerPositionRotation {
        x: Double,
        y: Double,
        z: Double,
        yaw: Float,
        pitch: Float,
        flags: Byte
    },
    PlayerRotation {
        yaw: Float,
        pitch: Float,
        flags: Byte
    },
    PlayerAction {
        status: VarInt,
        position: Position,
        face: Byte,
        sequence: VarInt
    },
    PlayerLoaded(),
    Punch(),
    SetHeldItem {
        slot: Short
    },
    UseItemOn {
        hand: VarInt,
        location: Position,
        face: VarInt,
        cursor_x: Float,
        cursor_y: Float,
        cursor_z: Float,
        inside_block: Bool,
        world_border: Bool,
        sequence: VarInt
    },
    UseItem {
        hand: VarInt,
        sequence: VarInt,
        yaw: Float,
        pitch: Float
    },

    // Clientbound
    BundleDelimiter(),
    AddEntity {
        id: VarInt,
        uuid: UUID,
        entity_type: VarInt,
        x: Double,
        y: Double,
        z: Double,
        velocity: LpVec3,
        pitch: UnsignedByte,
        yaw: UnsignedByte,
        head_yaw: UnsignedByte,
        data: VarInt
    },
    EntityAnimate {
        id: VarInt,
        animation: UnsignedByte
    },
    BlockChangedAcknowledge { sequence: VarInt },
    BlockAction {
        location: Position,
        id: UnsignedByte,
        parameter: UnsignedByte,
        block_type: VarInt
    },
    BlockUpdate {
        location: Position,
        id: VarInt
    },
    ChunkBiomes(), // I ain't doin allat
    KeepClientAlive { id: Long },
    EntityPosition {
        entity_id: VarInt,
        properties: VarInt,
    },
    EntityPositionRotation {
        entity_id: VarInt,
        properties: VarInt,
    },
    RotateHead {
        entity_id: VarInt,
        head_yaw: UnsignedByte
    },
    EntityMetadata(), // I ain't doin allat
    UpdateTime {
        world_age: Long,
        clocks: PrefixedArray<(VarInt, VarLong, Float, Float)>
    },
    SoundEffect { event: String,
        category: VarInt,
        x: Int,
        y: Int,
        z: Int,
        volume: Float,
        pitch: Float,
        seed: Long 
    }
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
            (Serverbound, 1) => Attack { entity_id: VarInt::decode(bytes)? },
            (Serverbound, 13) => ClientTickEnd(),
            (Serverbound, 28) => KeepServerAlive { id: Long::decode(bytes)? },
            (Serverbound, 30) => PlayerPosition {
                x:              Double::decode(bytes)?,
                y:              Double::decode(bytes)?,
                z:              Double::decode(bytes)?,
                flags:          Byte::decode(bytes)?
            },
            (Serverbound, 31) => PlayerPositionRotation {
                x:              Double::decode(bytes)?,
                y:              Double::decode(bytes)?,
                z:              Double::decode(bytes)?,
                yaw:            Float::decode(bytes)?,
                pitch:          Float::decode(bytes)?,
                flags:          Byte::decode(bytes)?
            },
            (Serverbound, 32) => PlayerRotation {
                yaw:            Float::decode(bytes)?,
                pitch:          Float::decode(bytes)?,
                flags:          Byte::decode(bytes)?
            },
            (Serverbound, 41) => PlayerAction {
                status:         VarInt::decode(bytes)?,
                position:       Position::decode(bytes)?,
                face:           Byte::decode(bytes)?,
                sequence:       VarInt::decode(bytes)?
            },
            (Serverbound, 44) => Punch(),
            // Why does the protocol skip 45?
            (Serverbound, 46) => Punch(),
            (Serverbound, 54) => SetHeldItem { slot: Short::decode(bytes)? },
            (Serverbound, 66) => UseItemOn { 
                hand:           VarInt::decode(bytes)?,
                location:       Position::decode(bytes)?,
                face:           VarInt::decode(bytes)?,
                cursor_x:       Float::decode(bytes)?,
                cursor_y:       Float::decode(bytes)?,
                cursor_z:       Float::decode(bytes)?,
                inside_block:   Bool::decode(bytes)?,
                world_border:   Bool::decode(bytes)?,
                sequence:       VarInt::decode(bytes)?
            },
            (Serverbound, 67) => UseItem {
                hand:           VarInt::decode(bytes)?,
                sequence:       VarInt::decode(bytes)?,
                yaw:            Float::decode(bytes)?,
                pitch:          Float::decode(bytes)?
            },
            (Clientbound, 0) => return Ok(parse_bundle(bytes)?),
            (Clientbound, 1) => {
                let add_entity = AddEntity { 
                    id:             VarInt::decode(bytes)?,
                    uuid:           UUID::decode(bytes)?,
                    entity_type:    VarInt::decode(bytes)?,
                    x:              Double::decode(bytes)?,
                    y:              Double::decode(bytes)?,
                    z:              Double::decode(bytes)?,
                    velocity:       LpVec3::decode(bytes)?,
                    pitch:          UnsignedByte::decode(bytes)?,
                    yaw:            UnsignedByte::decode(bytes)?,
                    head_yaw:       UnsignedByte::decode(bytes)?,
                    data:           VarInt::decode(bytes)?
                };

                add_entity
            },
            (Clientbound, 2) => EntityAnimate { 
                id:             VarInt::decode(bytes)?,
                animation:      UnsignedByte::decode(bytes)?
            },
            (Clientbound, 4) => BlockChangedAcknowledge { sequence: VarInt::decode(bytes)? },
            (Clientbound, 7) => BlockAction {
                location:       Position::decode(bytes)?,
                id:             UnsignedByte::decode(bytes)?,
                parameter:      UnsignedByte::decode(bytes)?,
                block_type:     VarInt::decode(bytes)?
            },
            (Clientbound, 8) => BlockUpdate {
                location:       Position::decode(bytes)?,
                id:             VarInt::decode(bytes)?
            },
            (Clientbound, 13) => ChunkBiomes(),
            (Clientbound, 45) => KeepClientAlive { id: Long::decode(bytes)? },
            (Clientbound, 54) => EntityPosition {
                entity_id: VarInt::decode(bytes)?,
                properties: VarInt::decode(bytes)?
            },
            (Clientbound, 55) => EntityPositionRotation {
                entity_id: VarInt::decode(bytes)?,
                properties: VarInt::decode(bytes)?
            },
            (Clientbound, 85) => RotateHead {
                entity_id: VarInt::decode(bytes)?,
                head_yaw: UnsignedByte::decode(bytes)?
            },
            (Clientbound, 101) => EntityMetadata(),
            (Clientbound, 115) => UpdateTime {
                world_age: Long::decode(bytes)?,
                clocks: PrefixedArray::<(VarInt, VarLong, Float, Float)>::decode(bytes)?
            },
            (Clientbound, 119) => SoundEffect {
                event: String::decode(bytes)?,
                category: VarInt::decode(bytes)?,
                x: Int::decode(bytes)?,
                y: Int::decode(bytes)?,
                z: Int::decode(bytes)?,
                volume: Float::decode(bytes)?,
                pitch: Float::decode(bytes)?,
                seed: Long::decode(bytes)?
            },
            _ => {
                println!("Unable to handle packet id {}", id);
                Err(DecodeError::MalformedBytes)?
            }
        };

        Ok(vec![PhasePayload::Play(payload)])
    }
}

// Thank you big pickle <3
fn parse_bundle(bytes: &mut &[u8]) -> Result<Vec<PhasePayload>, DecodeError> {
    let mut bundle: Vec<PhasePayload> = Vec::new();

    loop {
        let VarInt(length) = VarInt::decode(bytes)?;
        let length = length as usize;

        let all: &[u8] = *bytes;
        if all.len() < length {
            return Err(DecodeError::PrematureByteEnd);
        }
        let (mut frame, rest) = all.split_at(length);
        *bytes = rest;

        let id = VarInt::decode(&mut frame)?;

        if id.0 == 0 {
            return Ok(bundle);
        }


        if let Ok(mut payloads) = Payload::parse(&mut frame, Direction::Clientbound, id) {
            bundle.append(&mut payloads);
        }
    }
}
