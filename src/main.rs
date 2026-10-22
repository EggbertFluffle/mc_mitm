use pcap::{Capture, Device};

mod mc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let device = Device::from("lo");
    let device = Device::lookup()?
        .ok_or("no device available")?;
    println!("Using device: {}", device.name);

    let mut cap = Capture::from_device(device)?
        .immediate_mode(true)
        .timeout(0)
        .open()?;

    cap.filter("tcp port 25565 and tcp[tcpflags] & tcp-push != 0", true)?;

    let mut protocol_parser = mc::ProtocolParser{ phase: mc::Phase::Handshake };

    for raw_packet in cap.iter(mc::Codec) {
        let raw_packet = raw_packet?;

        if let Ok(packet) = protocol_parser.parse_packet(raw_packet) {
            println!("{packet:?}", )
        }

        // let packet = protocol_parser.parse_packet(raw_packet);
        // println!("{packet:?}");
    }

    Ok(())
}
