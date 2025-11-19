use glam::Vec3;
use shared::Player;
use wgpu::{BindGroupLayout, Device, Queue};

use crate::{
    engine::TexInfo,
    map::{ClientTile, Drawable},
};

pub struct ClientPlayer {
    pub id: String,
    pub tile: ClientTile,
    pub scale: f32,
    pub speed: f32,
}

impl ClientPlayer {
    pub fn new(
        device: &Device,
        layout: &BindGroupLayout,
        id: String,
        position: [f32; 3],
        tex_info: &TexInfo,
        scale: f32,
        speed: f32,
    ) -> Self {
        let tile: ClientTile = ClientTile::new(device, layout, position, &tex_info, scale);

        Self {
            id,
            tile,
            scale,
            speed,
        }
    }

    pub fn update_player(&mut self, queue: &Queue, player: Player) {
        self.speed = player.speed;
        self.tile.world_position = player.position;

        let iso_coords: [f32; 3] = [
            (self.tile.world_position[0] - self.tile.world_position[1]) as f32 * 0.5 * self.scale,
            (self.tile.world_position[0] + self.tile.world_position[1]) as f32 * 0.25 * self.scale
                + (self.tile.world_position[2] as f32 * 0.5 * self.scale),
            0.0,
        ];

        self.tile.translate(queue, Vec3::new(iso_coords[0], iso_coords[1], iso_coords[2]));
    }

    pub fn move_player(&mut self, queue: &Queue, direction: [f32; 3]) {
        let pos = [
            self.tile.world_position[0] + direction[0] * self.speed,
            self.tile.world_position[1] + direction[1] * self.speed,
            self.tile.world_position[2] + direction[2] * self.speed,
        ];

        let iso_coords: [f32; 3] = [
            (pos[0] - pos[1]) as f32 * 0.5 * self.scale,
            (pos[0] + pos[1]) as f32 * 0.25 * self.scale + (pos[2] as f32 * 0.5 * self.scale),
            0.0,
        ];

        self.tile.world_position = pos;
        self.tile.translate(
            queue,
            Vec3::new(iso_coords[0], iso_coords[1], iso_coords[2]),
        );
    }
}

impl Drawable for ClientPlayer {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        self.tile.render(render_pass);
    }
}
