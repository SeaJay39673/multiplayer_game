use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{TileType, generate_heightmap, map::Tile};


#[derive(Serialize, Deserialize, Clone)]
pub struct TileManager {
    pub tiles: BTreeMap<(i64, i64, i64), Tile>,
    pub position: [i64; 2],
    pub size: u8,
}

impl TileManager {
    pub fn new(position: [i64; 2]) -> Self {
        let scale = 0.25;
        let size: u8 = 4;
        let mut tiles:BTreeMap<(i64, i64, i64), Tile> = BTreeMap::new();
        
        if let Ok(height_map) = generate_heightmap(&position, size) {
            height_map.iter().for_each(|((x,y), z)| {
                let tile = Tile::new([*x, *y, *z], TileType::GrassBlock, scale);
                tiles.insert((*x, *y, *z), tile);
            });
        } else {
            for y in -(size as i64)..=size as i64 {
                for x in -(size as i64)..=size as i64 {
                    let tile = Tile::new([x, y, 0], TileType::GrassBlock, scale);
                    tiles.insert((x, y, 0), tile);
                }
            }
        }
        
        Self {
            tiles: tiles,
            position,
            size,
        }
    }
}