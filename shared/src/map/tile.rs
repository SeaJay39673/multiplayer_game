use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum TileType {
    GrassBlock,
    GrassSlopeL,
    GrassSlopeR,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tile {
    tile_type: TileType,
    world_position: [i64; 3],
    iso_position: [f32;2],
}

impl Tile {
    pub fn new(position: [i64; 3], tile_type: TileType, scale: f32) -> Self {
        let iso_coords: [f32; 2] = [
            (position[0] - position[1]) as f32 * 0.5 * scale,
            (position[0] + position[1]) as f32 * 0.25 * scale + (position[2] as f32 * 0.5 * scale),
        ];

        Self {
            tile_type,
            world_position: position,
            iso_position: iso_coords,
        }
    }
}