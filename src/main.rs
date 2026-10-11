use pcap::{Capture, Device};

mod mc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::lookup()?
        .ok_or("no device available")?;
    println!("Using device: {}", device.name);

    let mut cap = Capture::from_device(device)?.immediate_mode(true).open()?;

    cap.filter("tcp port 25565", true)?;

    for packet in cap.iter(mc::Codec) {
        let packet = packet?;
        let headers = packet.headers()?;

        println!("{headers:?}");
    }

    Ok(())
}
