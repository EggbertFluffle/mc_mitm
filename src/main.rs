use pcap::{Capture, Device};

mod mc;
mod web;

const PORT: u16 = 3000;
const TARGET_IP: &str = "18.42.101.96";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::lookup()?
        .ok_or("no device available")?;
    println!("Using device: {}", device.name);

    let mut cap = Capture::from_device(device)?
        .immediate_mode(true)
        .timeout(0)
        .open()?;

    cap.filter("tcp port 25565", true)?;

    let mut protocol_parser = mc::ProtocolParser::new(mc::Phase::Handshake);

    for raw_packet in cap.iter(mc::Codec) {
        let raw_packet = raw_packet?;

        if let Ok(packet) = protocol_parser.parse_packet(raw_packet) {
            if let Some(payload) = packet.payload.get(0) {
                if let mc::PhasePayload::Play(mc::play::Payload::ClientTickEnd()) = payload { } else {
                    println!("{:?}", payload);
                }
            }
        }

        // let packet = protocol_parser.parse_packet(raw_packet);
        // println!("{packet:?}");
    }

    web::run(PORT).await;

    Ok(())
}
