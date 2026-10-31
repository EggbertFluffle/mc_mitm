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

use crate::mc::Phase::Play;

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
    pub compression: Option<u32>
}

impl ProtocolParser {
    // parse_packet will return None if the packet was not complete and is expecting more bytes to
    // complete the packet
    pub fn parse_packet(&mut self, raw_packet: RawPacket) -> Result<Packet, DecodeError> {
        use etherparse::TransportHeader::Tcp;
        use Phase::*;
        use Direction::{ Serverbound, Clientbound };

        // Use etherparse to get packet headers and data start
        let headers = raw_packet.headers().map_err(|_| { DecodeError::PrematureByteEnd })?;
        let mut bytes: &[u8] = headers.payload.slice();

        let direction = if let Some(Tcp(tcp_header)) = headers.transport {
            // TODO: Don't use magic number
            if tcp_header.destination_port == 25565 { Serverbound } else { Clientbound }
        } else { panic!("Received packet without TCP headers") };

        let _ = VarInt::decode(&mut bytes)?; // Toss the length, we don't need that jawn

        let mut decompressed = Vec::new();

        let id = if self.compression.is_some() {
            let VarInt(data_length) = VarInt::decode(&mut bytes)?;

            if data_length == 0 {
                VarInt::decode(&mut bytes)?
            } else {
                let data_length = usize::try_from(data_length)
                    .map_err(|_| DecodeError::MalformedBytes)?;

                decompressed.resize(data_length, 0);

                let (uncompressed, status) = decompress_slice(
                    &mut decompressed,
                    bytes,
                    InflateConfig::default()
                );

                if status != zlib_rs::ReturnCode::Ok || uncompressed.len() != data_length {
                    return Err(DecodeError::MalformedBytes);
                }

                let len = uncompressed.len();
                decompressed.truncate(len);
                bytes = &decompressed;

                VarInt::decode(&mut bytes)?
            }
        } else {
            VarInt::decode(&mut bytes)?
        };

        let packet = Packet { 
            id,
            direction, 
            phase: self.phase,
            payload: match self.phase {
                Handshake => handshake::Payload::parse(&mut bytes, direction, id)?,
                Status => status::Payload::parse(&mut bytes, direction, id)?,
                Login => login::Payload::parse(&mut bytes, direction, id)?,
                Configuration => configuration::Payload::parse(&mut bytes, direction, id)?,
                Play => play::Payload::parse(&mut bytes, direction, id)?
            }
        };

        if let Some(payload) = &packet.payload.get(0) {
            self.phase = match payload {
                PhasePayload::Handshake(handshake::Payload::Intention { intent, .. }) => match intent.0 {
                    1 => Status,
                    2..=3 => Login,
                    _ => Err(DecodeError::MalformedBytes)?
                },
                PhasePayload::Status(status::Payload::PingResponse { .. }) => Handshake,
                PhasePayload::Login(login::Payload::LoginAcknowledged { .. }) => Configuration,
                PhasePayload::Configuration(configuration::Payload::FinishConfigurationAcknowledge { .. }) => Play,
                _ => self.phase
            };

            return match payload {
                PhasePayload::Login(login::Payload::EncryptionRequest())
                    => panic!("Encryption is not supported"),
                PhasePayload::Login(login::Payload::EncryptionResponse())
                    => panic!("Encryption is not supported"),
                PhasePayload::Login(login::Payload::SetCompression { threshold }) => {
                    self.compression = if threshold.0 >= 0 { Some(threshold.0 as u32) } else { None };

                    Ok(packet)
                }
                _ => Ok(packet)
            };
        }
        
        Ok(packet)
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

        let filter = format!("tcp port 25565");
        cap.filter(&filter, true)
            .expect("Failed to parse pcap filter");

        for raw_packet in cap.iter(Codec) {
            let raw_packet = match raw_packet {
                Err(_) => continue,
                Ok(rp) => rp
            };

            if let Ok(packet) = self.protocol_parser.parse_packet(raw_packet) {
                for payload in &packet.payload {
                    self.change_state(payload);
                }
            }
        }
    }

    fn change_state(&mut self, phase_payload: &PhasePayload) {
        use PhasePayload::*;

        // Only play packets carry the state we track; don't take the write lock
        // for every packet on the wire.
        let Play(payload) = phase_payload else { return; };

        use play::Payload::*;

        let mut game_state = self.game_state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match payload {
            PlayerPosition { x, z, .. } => {
                game_state.player.x = *x;
                game_state.player.z = *z;
            },
            PlayerRotation { yaw, .. } => {
                game_state.player.yaw = *yaw;
            },
            PlayerPositionRotation { x, z, yaw, .. } => {
                game_state.player.x = *x;
                game_state.player.z = *z;
                game_state.player.yaw = *yaw;
            },
            _ => { }
        };
    }
}
