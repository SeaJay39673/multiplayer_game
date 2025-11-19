use std::collections::BTreeMap;

use shared::TileManager;
use wgpu::{BindGroupLayout, Device};

use crate::{engine::{TEXTURE_MAP}, map::{ClientTile, Drawable}};



pub struct ClientTileManager {
    position: [i64; 2],
    tiles: BTreeMap<(i64, i64, i64), ClientTile>,
}

impl ClientTileManager {
    pub fn from_server(value: TileManager, device: &Device, layout: &BindGroupLayout, size: u8, scale: f32) -> Self {
        let mut tiles: BTreeMap<(i64, i64, i64), ClientTile> = BTreeMap::new();

        let textures = if let Ok(textures) = TEXTURE_MAP.read() {
            textures
        } else {
            panic!("Could not get TEXTURE_MAP for reading");
        };

        let tex_info = if let Some(tex_info) = textures.get(&shared::TileType::GrassBlock) {
            tex_info
        } else {
            panic!("Could not get TexInfo");
        };
        
        value.tiles.iter().for_each(|((x, y, z), tile)| {
            let client_tile: ClientTile = ClientTile::new(device, layout, [*x as f32,*y as f32,*z as f32], &tex_info, scale);
            tiles.insert((*z, -y, -x), client_tile);
        });

        Self {
            tiles,
            position: value.position,
        }
    }
}

impl Drawable for ClientTileManager {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        self.tiles.iter().for_each(|((z, ny, nx), tile)| {
            tile.render(render_pass);
        });
    }
}