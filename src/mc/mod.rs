mod types;
pub mod state;

pub mod handshake;
pub mod status;
pub mod login;
pub mod configuration;
pub mod play;

use std::sync::{Arc, RwLock};

use pcap::{Capture, Device};

use etherparse::{PacketHeaders, err::packet::SliceError};
use zlib_rs::{decompress_slice, InflateConfig};

use types::{ VarInt };
use state::GameState;

use crate::mc::state::Entity;

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    MalformedBytes,
    PrematureByteEnd
}

// Types that "consume" parts of a packet as they are parsed
// They should do this by modifying the reference to the bytes slice
pub trait Decode {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> where Self: Sized;
}

pub trait Parse {
    fn parse(
        bytes: &mut &[u8],
        direction: Direction,
        id: VarInt
    ) -> Result<Vec<PhasePayload>, DecodeError>;
}

// 5 Phases of the protocol
// Dictate how packet IDs and directions are interpreted
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum Phase {
    #[default] Handshake,
    Status,
    Login,
    Configuration,
    Play
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { Serverbound, Clientbound }

#[allow(unused)]
#[derive(Debug)]
pub struct Packet {
    pub id: VarInt,
    pub direction: Direction,
    pub phase: Phase,
    pub payload: Vec<PhasePayload>
}

#[derive(Debug, Clone, Default)]
pub struct ProtocolParser {
    pub phase: Phase,
    pub compression: Option<u32>,
    streams: [Vec<u8>; 2],
    next_seq: [Option<u32>; 2]
}

impl ProtocolParser {
    pub fn parse_packets(&mut self, raw_packet: RawPacket) -> Vec<Packet> {
        use etherparse::TransportHeader::Tcp;
        use Direction::{ Serverbound, Clientbound };

        // Use etherparse to get packet headers and data start
        let headers = match raw_packet.headers() {
            Ok(headers) => headers,
            Err(_) => return Vec::new()
        };

        let tcp_header = if let Some(Tcp(tcp_header)) = headers.transport {
            tcp_header
        } else {
            return Vec::new();
        };

        // TODO: Don't use magic number
        let direction = if tcp_header.destination_port == 25565 { Serverbound } else { Clientbound };

        // A new or reset connection invalidates any partially buffered packet.
        if tcp_header.syn || tcp_header.rst {
            let index = Self::stream_index(direction);
            self.streams[index].clear();
            self.next_seq[index] = if tcp_header.syn {
                Some(tcp_header.sequence_number.wrapping_add(1))
            } else {
                None
            };

            return Vec::new();
        }

        let bytes = headers.payload.slice();
        if bytes.is_empty() {
            return Vec::new();
        }

        eprintln!("SEGMENT: dir={:?} syn={} rst={} len={}", direction, tcp_header.syn, tcp_header.rst, bytes.len());
        self.push_segment(direction, tcp_header.sequence_number, bytes)
    }

    fn push_segment(&mut self, direction: Direction, sequence: u32, bytes: &[u8]) -> Vec<Packet> {
        let index = Self::stream_index(direction);
        let length = bytes.len() as u32;

        let skip = match self.next_seq[index] {
            None => {
                self.next_seq[index] = Some(sequence.wrapping_add(length));
                0
            }
            Some(expected) => {
                let difference = sequence.wrapping_sub(expected);

                if difference == 0 {
                    self.next_seq[index] = Some(expected.wrapping_add(length));
                    0
                } else if difference < 0x8000_0000 {
                    // drop a partial packet and resume at the start of this segment
                    self.streams[index].clear();
                    self.next_seq[index] = Some(sequence.wrapping_add(length));
                    0
                } else {
                    // Behind where we expected to be: retransmission or duplicate.
                    let overlap = expected.wrapping_sub(sequence);
                    if overlap >= length {
                        return Vec::new();
                    }

                    self.next_seq[index] = Some(expected.wrapping_add(length - overlap));
                    overlap as usize
                }
            }
        };

        self.streams[index].extend_from_slice(&bytes[skip..]);
        self.drain_stream(direction)
    }

    fn stream_index(direction: Direction) -> usize {
        match direction {
            Direction::Serverbound => 0,
            Direction::Clientbound => 1
        }
    }

    // Peek a VarInt without consuming it. Ok(None) means more bytes are needed.
    fn peek_varint(bytes: &[u8]) -> Result<Option<(VarInt, usize)>, DecodeError> {
        let mut remaining = bytes;

        match VarInt::decode(&mut remaining) {
            Ok(value) => Ok(Some((value, bytes.len() - remaining.len()))),
            Err(DecodeError::PrematureByteEnd) => Ok(None),
            Err(error) => Err(error)
        }
    }

    fn drain_stream(&mut self, direction: Direction) -> Vec<Packet> {
        let index = Self::stream_index(direction);
        let mut packets = Vec::new();

        loop {
            let (length, length_bytes) = match Self::peek_varint(&self.streams[index]) {
                Ok(Some(value)) => value,
                Ok(None) => break,
                Err(_) => {
                    self.streams[index].clear();
                    break;
                }
            };

            let length = match usize::try_from(length.0) {
                Ok(length) => length,
                Err(_) => {
                    self.streams[index].clear();
                    break;
                }
            };

            let total = match length_bytes.checked_add(length) {
                Some(total) => total,
                None => {
                    self.streams[index].clear();
                    break;
                }
            };

            if self.streams[index].len() < total {
                break;
            }

            let frame = self.streams[index][length_bytes..total].to_vec();
            self.streams[index].drain(..total);

            if let Some(packet) = self.parse_frame(direction, &frame) {
                eprintln!("GOT PACKET: phase={:?} dir={:?} id={} payloads={}", packet.phase, packet.direction, packet.id.0, packet.payload.len());
                packets.push(packet);
            } else {
                eprintln!("PARSE_FRAME_RETURNED_NONE for frame of len {}", frame.len());
            }
        }

        packets
    }

    fn decode_frame(&self, frame: &[u8]) -> Result<Vec<u8>, DecodeError> {
        if self.compression.is_none() {
            return Ok(frame.to_vec());
        }

        let mut compressed = frame;
        let VarInt(data_length) = VarInt::decode(&mut compressed)?;

        if data_length == 0 {
            return Ok(compressed.to_vec());
        }

        let data_length = usize::try_from(data_length)
            .map_err(|_| DecodeError::MalformedBytes)?;

        let mut decompressed = vec![0; data_length];

        let (uncompressed, status) = decompress_slice(
            &mut decompressed,
            compressed,
            InflateConfig::default()
        );

        if status != zlib_rs::ReturnCode::Ok || uncompressed.len() != data_length {
            return Err(DecodeError::MalformedBytes);
        }

        let len = uncompressed.len();
        decompressed.truncate(len);

        Ok(decompressed)
    }

    fn parse_frame(&mut self, direction: Direction, frame: &[u8]) -> Option<Packet> {
        use Phase::*;

        let data = self.decode_frame(frame).ok()?;
        let mut bytes: &[u8] = &data;

        let id = VarInt::decode(&mut bytes).ok()?;

        let payload_res = match self.phase {
            Handshake => handshake::Payload::parse(&mut bytes, direction, id),
            Status => status::Payload::parse(&mut bytes, direction, id),
            Login => login::Payload::parse(&mut bytes, direction, id),
            Configuration => configuration::Payload::parse(&mut bytes, direction, id),
            Play => play::Payload::parse(&mut bytes, direction, id)
        };
        let payload = match payload_res {
            Ok(p) => p,
            Err(e) => {
                eprintln!("PAYLOAD_PARSE_ERR: phase={:?} dir={:?} id={} err={:?} bytes_left={}", self.phase, direction, id.0, e, bytes.len());
                return None;
            }
        };
        let packet = Packet {
            id,
            direction,
            phase: self.phase,
            payload,
        };

        if let Some(payload) = packet.payload.first() {
            self.phase = match payload {
                PhasePayload::Handshake(handshake::Payload::Intention { intent, .. }) => match intent.0 {
                    1 => Status,
                    2..=3 => Login,
                    _ => return None
                },
                PhasePayload::Status(status::Payload::PingResponse { .. }) => Handshake,
                PhasePayload::Login(login::Payload::LoginAcknowledged { .. }) => Configuration,
                PhasePayload::Configuration(configuration::Payload::FinishConfigurationAcknowledge { .. }) => Play,
                _ => self.phase
            };

            match payload {
                PhasePayload::Login(login::Payload::EncryptionRequest())
                    => panic!("Encryption is not supported"),
                PhasePayload::Login(login::Payload::EncryptionResponse())
                    => panic!("Encryption is not supported"),
                PhasePayload::Login(login::Payload::SetCompression { threshold }) => {
                    self.compression = if threshold.0 >= 0 { Some(threshold.0 as u32) } else { None };
                }
                _ => { }
            }
        }

        Some(packet)
    }
}

#[derive(Debug)]
pub enum PhasePayload {
    Handshake(handshake::Payload),
    Status(status::Payload),
    Login(login::Payload),
    Configuration(configuration::Payload),
    Play(play::Payload)
}

pub struct RawPacket(Box<[u8]>);

impl RawPacket {
    pub fn headers<'a>(&'a self) -> Result<PacketHeaders<'a>, SliceError> {
        PacketHeaders::from_ethernet_slice(&self.0)
    }
}

pub struct Codec;

impl pcap::PacketCodec for Codec {
    type Item = RawPacket;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        RawPacket(packet.data.into())
    }
}

pub struct Bridge {
    game_state: Arc<RwLock<GameState>>,
    protocol_parser: ProtocolParser
}

impl Bridge {
    pub fn new(game_state: Arc<RwLock<GameState>>) -> Bridge {
        Bridge { 
            game_state,
            protocol_parser: ProtocolParser::default(),
        }
    }

    pub fn run(&mut self) {
        let device = Device::lookup()
            .expect("No device available")
            .expect("No suitable devices present");
        println!("Using device: {}", device.name);

        let mut cap = Capture::from_device(device)
            .expect("Unable to capture from device")
            .immediate_mode(true)
            .timeout(0)
            .open()
            .expect("Unable to open device for capturing");

        let filter = "tcp port 25565";
        cap.filter(filter, true)
            .expect("Failed to parse pcap filter");

        for raw_packet in cap.iter(Codec) {
            let raw_packet = match raw_packet {
                Err(_) => continue,
                Ok(rp) => rp
            };

            for packet in self.protocol_parser.parse_packets(raw_packet) {
                for payload in &packet.payload {
                    self.change_state(payload);
                }
            }
        }
    }

    fn change_state(&mut self, phase_payload: &PhasePayload) {
        use PhasePayload::*;

        let Play(payload) = phase_payload else { return; };

        use play::Payload::*;

        let mut game_state = self.game_state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match payload {
            Attack { id } => {
                game_state.action_log.push(format!("Player has attacked {}", id.0));
            },
            PlayerPosition { x, z, .. } => {
                game_state.player.x = *x;
                game_state.player.z = *z;
            },
            PlayerRotation { yaw, .. } => {
                game_state.player.yaw = yaw.rem_euclid(360.0);
            },
            PlayerPositionRotation { x, z, yaw, .. } => {
                game_state.player.x = *x;
                game_state.player.z = *z;
                game_state.player.yaw = yaw.rem_euclid(360.0);
            },
            AddEntity { id, entity_type, x, z, yaw, .. } => {
                game_state.entities.entry(id.0)
                    .or_insert(Entity {
                        x: *x,
                        z: *z,
                        yaw: *yaw,
                        entity_type: entity_type.0
                    });
            },
            EntityPosition { id, steps, .. } => {
                game_state.entities.entry(id.0)
                    .and_modify(|entity| {
                        for s in steps {
                            entity.x += s.1 as f64;
                            entity.z += s.3 as f64;
                        }
                    });
            },
            EntityPositionRotation { id, steps, yaw, .. } => {
                game_state.entities.entry(id.0)
                    .and_modify(|entity| {
                        for s in steps {
                            entity.x += s.1 as f64;
                            entity.z += s.3 as f64;
                        }
                        entity.yaw = *yaw;
                    });
            },
            EntityRotation { id, yaw, .. } => {
                game_state.entities.entry(id.0)
                    .and_modify(|entity| {
                        entity.yaw = *yaw;
                    });
            },
            RemoveEntities { ids } => {
                for ele in ids {
                    _ = game_state.entities.remove(&ele.0);
                }
            },
            UpdateTime { world_age, .. } => {
                game_state.timestamp = *world_age;
            },
            OpenScreen { window_type, .. } => {
                game_state.action_log.push(format!(
                    "Player has opened up a {}",
                    match window_type.0 {
                        2   => "chest",
                        5   => "double chest",
                        6   => "dispenser",
                        7   => "crafter",
                        8   => "anvil",
                        9   => "beacon",
                        10  => "blast_furnace",
                        11  => "brewing_stand",
                        12  => "crafting",
                        13  => "enchantment",
                        14  => "furnace",
                        15  => "grindstone",
                        16  => "hopper",
                        17  => "lectern",
                        18  => "loom",
                        19  => "merchant",
                        20  => "shulker_box",
                        21  => "smithing",
                        22  => "smoker",
                        23  => "cartography_table",
                        24  => "stonecutter",
                        _   => "unknown"
                    }
                ));
            },
            CombatDeath { .. } => {
                game_state.action_log.push("Player died".to_string());
            },
            Chat { message, .. } => {
                game_state.action_log.push(format!("Sent chat: \"{}\"", message.as_str()));
            },
            CloseContainer { .. } => {
                game_state.action_log.push("Closed contianer".to_string());
            },
            RenameItem { name } => {
                game_state.action_log.push(format!("Renamed item: \"{}\"", name.as_str()));
            },
            UpdateSign { line1, line2, line3, line4, ..} => {
                game_state.action_log.push( format!( "Updated sign: \"{}\", \"{}\", \"{}\", \"{}\"", line1.as_str(), line2.as_str(), line3.as_str(), line4.as_str()));
            },
            _ => { }
        };
    }
}
