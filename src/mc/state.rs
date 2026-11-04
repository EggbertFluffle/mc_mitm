use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Entity {
    pub x: f64,
    pub z: f64,
    pub yaw: u8,
    pub entity_type: i32
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Player {
    pub x: f64,
    pub z: f64,
    pub yaw: f32
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GameState {
    pub player: Player,
    pub entities: HashMap<i32, Entity>
}
