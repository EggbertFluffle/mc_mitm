mod mc;
mod web;

const PORT: u16 = 3000;
const TARGET_IP: &str = "18.42.101.96";

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Spawn Web Server on Separate thread
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all().build().unwrap()
            .block_on(web::run(PORT));
    });

    // Capture owns main thread.
    mc::run()?;
    Ok(())
}
