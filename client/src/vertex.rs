use std::mem;

use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexFloat32 {
    pub position: [f32; 2],
    pub uv: [f32; 2], 
}

impl VertexFloat32 {
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout { 
            array_stride: mem::size_of::<VertexFloat32>() as BufferAddress, 
            step_mode: VertexStepMode::Vertex, 
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2
                },
                VertexAttribute {
                    offset: (mem::size_of::<[f32; 2]>()) as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2
                }
            ],
        }
    }
}