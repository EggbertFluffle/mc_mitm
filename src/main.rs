use std::sync::{Arc, RwLock};

use crate::mc::{Bridge, state::GameState};

mod mc;
mod web;

const WEB_PORT: u16 = 3000;
const SERVER_PORT: u16 = 25565;
const CLIENT_IP: &str = "18.42.101.96";
const ROUTER_IP: &str = "18.42.101.96";

fn main() {
    let game_state = Arc::new(RwLock::new(GameState::default()));

    // Spawn Web Server on Separate thread
    let web_state = Arc::clone(&game_state);
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all().build().unwrap()
            .block_on(web::run(WEB_PORT, web_state));
    });

    // Capture owns main thread.
    let mut bridge = Bridge::new(game_state);
    bridge.run();
}
