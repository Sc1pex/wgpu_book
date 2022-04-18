#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    // Required because rust sees the result of vertex_attr_array as a temporary value
    // so it can't be returned from a function
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,

            // This is quite verbose so we use the vertex_attr_array macro
            // attributes: &[
            //     wgpu::VertexAttribute {
            //         format: wgpu::VertexFormat::Float32x3,
            //         offset: 0,
            //         shader_location: 0,
            //     },
            //     wgpu::VertexAttribute {
            //         format: wgpu::VertexFormat::Float32x3,
            //         offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            //         shader_location: 1,
            //     },
            // ],
            attributes: &Self::ATTRIBS,
        }
    }
}
