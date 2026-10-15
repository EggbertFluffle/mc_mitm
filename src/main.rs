use etherparse::{ TransportHeader::{ Tcp } };

use pcap::{Capture, Device};

use crate::mc::Decode;

mod mc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::lookup()?
        .ok_or("no device available")?;
    println!("Using device: {}", device.name);

    let mut cap = Capture::from_device(device)?.immediate_mode(true).open()?;

    cap.filter("tcp port 25565 and tcp[tcpflags] & tcp-push != 0", true)?;

    let protocol_state = mc::ProtocolState::Handshaking;

    for packet in cap.iter(mc::Codec) {
        let packet = packet?;
        let headers = packet.headers()?;
        let payload = headers.payload;
        let mut payload = payload.slice();

        if let Some(Tcp(header)) = headers.transport {
            let bytes = &mut payload;

            println!("Before handling: {}", bytes.len());
            let handshake = mc::handshake::Handshake::decode(bytes);
            println!("After handling: {}", bytes.len());

            println!("{handshake:?}");
        }
    }

    Ok(())
}
