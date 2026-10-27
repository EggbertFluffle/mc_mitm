use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Entity {
    pub id: i32,
    pub x: f64,
    pub z: f64,
    pub entity_type: &'static str,
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
    pub entities: Vec<Entity>
}
