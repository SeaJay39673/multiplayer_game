use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: String,
    pub position: [f32; 3],
    pub speed: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerMessage {
    pub id: String, 
    pub message: String
}