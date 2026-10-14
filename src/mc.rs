use etherparse::{PacketHeaders, PayloadSlice, err::packet::SliceError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    // Can likely switch this to a reference as long 
    // as we drop the reference after each loop iteration
    pub raw: Box<[u8]>
}

impl Packet {
    pub fn headers<'a>(&'a self) -> Result<PacketHeaders<'a>, SliceError> {
        PacketHeaders::from_ethernet_slice(&self.raw)
    }
}

pub struct Codec;
impl pcap::PacketCodec for Codec {
    type Item = Packet;

    fn decode(&mut self, packet: pcap::Packet<'_>) -> Self::Item {
        Packet {
            raw: packet.data.into()
        }
    }
}
