
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ]
        }
    }

    /// Generate triangle indices using a triangle-fan from the provided vertex slice.
    /// For n vertices returns (0,1,2),(0,2,3),... as u16 indices.
    pub fn generate_fan_indices(vertices: &[Vertex]) -> Vec<u16> {
        let n = vertices.len();
        if n < 3 {
            return Vec::new();
        }
        let mut indices = Vec::with_capacity((n - 2) * 3);
        for i in 1..(n - 1) {
            indices.push(0u16);
            indices.push(i as u16);
            indices.push((i + 1) as u16);
        }
        indices
    }
}

