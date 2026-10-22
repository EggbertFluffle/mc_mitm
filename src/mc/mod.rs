use etherparse::{PacketHeaders, err::packet::SliceError};

use types::{ VarInt };

mod types;
pub mod handshake;
pub mod status;
pub mod login;
pub mod configuration;
pub mod play;

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
    ) -> Result<PhasePayload, DecodeError>;
}

// 5 Phases of the protocol
// Dictate how pcaket IDs and directions are interpreted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Handshake,
    Status,
    Login,
    Configuration,
    Play
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { Serverbound, Clientbound }

#[derive(Debug)]
pub struct Packet {
    pub length: VarInt,
    pub id: VarInt,
    pub direction: Direction,
    pub phase: Phase,
    pub payload: PhasePayload
}

#[derive(Debug, Clone)]
pub struct ProtocolParser {
    pub phase: Phase
}

impl ProtocolParser {
    pub fn parse_packet(&mut self, raw_packet: RawPacket) -> Result<Packet, DecodeError> {
        use etherparse::TransportHeader::Tcp;
        use Phase::*;
        use Direction::{ Serverbound, Clientbound };

        // Use etherparse to get packet headers and data start
        let headers = raw_packet.headers().map_err(|_| { DecodeError::PrematureByteEnd })?;
        let mut bytes = &mut headers.payload.slice();

        // Parse and consume length and id from packet bytes
        let length = VarInt::decode(&mut bytes)?;
        let id = VarInt::decode(&mut bytes)?;

        let direction = if let Some(Tcp(tcp_header)) = headers.transport {
            // TODO: Don't use magic number
            if tcp_header.destination_port == 25565 { Serverbound } else { Clientbound }
        } else { panic!("Received packet without TCP headers") };


        let packet = Packet { 
            length,
            id,
            direction, 
            phase: self.phase,
            payload: match self.phase {
                Handshake => handshake::Payload::parse(&mut bytes, direction, id)?,
                Status => status::Payload::parse(&mut bytes, direction, id)?,
                Login => login::Payload::parse(&mut bytes, direction, id)?,
                Configuration => configuration::Payload::parse(&mut bytes, direction, id)?,
                Play => play::Payload::parse(&mut bytes, direction, id)?,
                _ => handshake::Payload::parse(&mut bytes, direction, id)?,
            }
        };

        self.phase = match &packet.payload {
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

        return match &packet.payload {
            PhasePayload::Login(login::Payload::EncryptionRequest())
                => panic!("Encryption is not supported"),
            PhasePayload::Login(login::Payload::EncryptionResponse())
                => panic!("Encryption is not supported"),
            PhasePayload::Login(login::Payload::SetCompression{ .. })
                => panic!("Compression is not supported"),
            _ => Ok(packet)
        };
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
