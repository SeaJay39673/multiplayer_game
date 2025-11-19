use glam::{Mat4, Vec3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::dpi::Position;

pub struct Camera {
    position: Vec3,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    camera_buffer: Buffer,
    bind_group: BindGroup,
    scale: f32,
}

impl Camera {
    pub fn new(
        device: &Device,
        layout: &BindGroupLayout,
        position: Vec3,
        target: Vec3,
        aspect_ratio: f32,
        scale: Option<f32>,
    ) -> Self {
        let scale = if let Some(scale) = scale { scale } else { 0.25 };

        let view_matrix = Mat4::look_at_rh(position, target, Vec3::Y);
        let projection_matrix = Mat4::orthographic_rh(
            -5.0 * aspect_ratio * scale,
            5.0 * aspect_ratio * scale,
            -5.0 * scale,
            5.0 * scale,
            0.1,
            100.0,
        );
        let vp_matrix = projection_matrix * view_matrix;
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(vp_matrix.as_ref()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Self {
            position,
            view_matrix,
            projection_matrix,
            camera_buffer,
            bind_group,
            scale,
        }
    }

    pub fn update(&mut self, queue: &Queue, position: Vec3) {
        self.position = position + Vec3::new(0.0, 10.0, 10.0);
        let target = position;
        self.view_matrix = Mat4::look_at_rh(self.position, target, Vec3::Y);
        let vp_matrix = self.projection_matrix * self.view_matrix;
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(vp_matrix.as_ref()),
        );
    }

    pub fn update_projection(&mut self, aspect_ratio: f32) {
        self.projection_matrix = Mat4::orthographic_rh(
            -5.0 * aspect_ratio * self.scale,
            5.0 * aspect_ratio * self.scale,
            -5.0 * self.scale,
            5.0 * self.scale,
            0.1,
            100.0,
        );
    }
}

impl Drawable for Camera {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
    }
}
