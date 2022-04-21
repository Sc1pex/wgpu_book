pub struct Instance {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];

    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        glam::Mat4::from_rotation_translation(self.rotation, self.position).to_cols_array_2d()
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
