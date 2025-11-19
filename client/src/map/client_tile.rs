use glam::{Mat4, Vec3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer, BufferUsages, Device, Queue, RenderPass, util::{BufferInitDescriptor, DeviceExt}
};

use crate::{engine::TexInfo, vertex::VertexFloat32};

pub struct ClientTile {
    pub world_position: [f32; 3],
    pub iso_position: [f32; 2],

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    transform: Mat4,
    transform_buffer: Buffer,
    bind_group: BindGroup,
}

impl ClientTile {
    pub fn new(
        device: &Device,
        layout: &BindGroupLayout,
        world_position: [f32; 3],
        tex_info: &TexInfo,
        scale: f32,
    ) -> Self {
        let half_size = scale / 2.0;
        let indices: &[u16] = &[0, 1, 2, 0, 2, 3];

        let vertices = [
            VertexFloat32 {
                position: [-1.0 * half_size, -1.0 * half_size],
                uv: tex_info.map_uv([0.0, 1.0]),
            },
            VertexFloat32 {
                position: [1.0 * half_size, -1.0 * half_size],
                uv: tex_info.map_uv([1.0, 1.0]),
            },
            VertexFloat32 {
                position: [1.0 * half_size, 1.0 * half_size],
                uv: tex_info.map_uv([1.0, 0.0]),
            },
            VertexFloat32 {
                position: [-1.0 * half_size, 1.0 * half_size],
                uv: tex_info.map_uv([0.0, 0.0]),
            },
        ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let iso_coords: [f32; 2] = [
            (world_position[0] - world_position[1]) as f32 * 0.5 * scale,
            (world_position[0] + world_position[1]) as f32 * 0.25 * scale
                + (world_position[2] as f32 * 0.5 * scale),
        ];

        let transform = Mat4::IDENTITY * Mat4::from_translation(Vec3::new(iso_coords[0], iso_coords[1], 0.0));

        let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(transform.as_ref()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Shape Bind Group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&tex_info.texture.view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&tex_info.texture.sampler),
                },
            ],
        });

        Self {
            world_position,
            iso_position: iso_coords,
            vertex_buffer,
            index_buffer,
            transform,
            transform_buffer,
            bind_group,
        }
    }

    pub fn translate(&mut self, queue: &Queue, direction: Vec3) {
        self.iso_position = [direction.x, direction.y];
        self.transform = Mat4::from_translation(direction);
        queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(self.transform.as_ref()),
        );
    }
}

pub trait Drawable {
    fn render(&self, render_pass: &mut RenderPass);
}

impl Drawable for ClientTile {
    fn render(&self, render_pass: &mut RenderPass) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
