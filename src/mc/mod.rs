use etherparse::{PacketHeaders, err::packet::SliceError};

mod types;
pub mod handshake;

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    MalformedBytes,
    PrematureByteEnd
}

pub trait Decode {
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodeError> where Self: Sized;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProtocolState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketCapture {
    pub raw: Box<[u8]>
}

impl PacketCapture {
    pub fn headers<'a>(&'a self) -> Result<PacketHeaders<'a>, SliceError> {
        PacketHeaders::from_ethernet_slice(&self.raw)
    }
}

pub struct Codec;

impl pcap::PacketCodec for Codec {
    type Item = PacketCapture;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        PacketCapture {
            raw: packet.data.into()
        }
    }
}
