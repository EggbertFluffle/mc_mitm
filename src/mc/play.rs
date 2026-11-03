use crate::mc::{ Decode, DecodeError, Direction, Parse, PhasePayload };
use crate::mc::types::*;

#[allow(unused)]
#[derive(Debug)]
pub enum Payload {
    // Serverbound
    Attack { id: VarInt },
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
        id: VarInt,
        on_ground: bool,
        steps: Vec<(VarInt, f32, f32, f32)>
    },
    EntityPositionRotation {
        id: VarInt,
        on_ground: bool,
        steps: Vec<(VarInt, f32, f32, f32)>,
        yaw: UnsignedByte,
        pitch: UnsignedByte
    },
    EntityRotation {
        id: VarInt,
        on_ground: bool,
        yaw: UnsignedByte,
        pitch: UnsignedByte
    },
    RemoveEntities {
        ids: PrefixedArray<VarInt>,
    },
    RotateHead {
        id: VarInt,
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
            (Serverbound, 1) => Attack { id: VarInt::decode(bytes)? },
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
            // Bundles are just ordinary length-prefixed packets between two delimiter packets.
            // Framing is handled by ProtocolParser, so the delimiter itself has no payload.
            (Clientbound, 0) => BundleDelimiter(),
            (Clientbound, 1) => AddEntity { 
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
            (Clientbound, 54) => {
                let id = VarInt::decode(bytes)?;
                let (on_ground, step_count) = {
                    let VarInt(properties) = VarInt::decode(bytes)?;
                    (properties & 1 == 1, properties >> 1)
                };

                let steps: Vec<(VarInt, f32, f32, f32)> = if step_count == 0 {
                    vec![(
                        VarInt(0),
                        (Short::decode(bytes)? as f32) / 4096.0,
                        (Short::decode(bytes)? as f32) / 4096.0,
                        (Short::decode(bytes)? as f32) / 4096.0,
                    )]
                } else {
                    let mut out = Vec::new();
                    for _ in 0..step_count {
                        out.push((
                            VarInt::decode(bytes)?,
                            (Short::decode(bytes)? as f32) / 4096.0,
                            (Short::decode(bytes)? as f32) / 4096.0,
                            (Short::decode(bytes)? as f32) / 4096.0,
                        ));
                    };
                    out
                };

                EntityPosition {
                    id,
                    on_ground,
                    steps
                }
            },
            (Clientbound, 55) => {
                let id = VarInt::decode(bytes)?;
                let (on_ground, step_count) = {
                    let VarInt(properties) = VarInt::decode(bytes)?;
                    (properties & 1 == 1, properties >> 1)
                };

                let steps: Vec<(VarInt, f32, f32, f32)> = if step_count == 0 {
                    vec![(
                        VarInt(0),
                        (Short::decode(bytes)? as f32) / 4096.0,
                        (Short::decode(bytes)? as f32) / 4096.0,
                        (Short::decode(bytes)? as f32) / 4096.0,
                    )]
                } else {
                    let mut out = Vec::new();
                    for _ in 0..step_count {
                        out.push((
                            VarInt::decode(bytes)?,
                            (Short::decode(bytes)? as f32) / 4096.0,
                            (Short::decode(bytes)? as f32) / 4096.0,
                            (Short::decode(bytes)? as f32) / 4096.0,
                        ));
                    };
                    out
                };

                EntityPositionRotation {
                    id,
                    on_ground,
                    steps,
                    yaw: UnsignedByte::decode(bytes)?,
                    pitch: UnsignedByte::decode(bytes)?
                }
            },
            (Clientbound, 57) => EntityRotation {
                id: VarInt::decode(bytes)?,
                on_ground: Bool::decode(bytes)?,
                yaw: UnsignedByte::decode(bytes)?,
                pitch: UnsignedByte::decode(bytes)?,
            },
            (Clientbound, 78) => RemoveEntities {
                ids: PrefixedArray::<VarInt>::decode(bytes)?
            },
            (Clientbound, 85) => RotateHead {
                id: VarInt::decode(bytes)?,
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
            _ => Err(DecodeError::MalformedBytes)?
        };

        Ok(vec![PhasePayload::Play(payload)])
    }
}

