pub struct Instance {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];

    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        // println!("{:?}", glam::Mat4::from_translation(self.position));
        // glam::Mat4::from_quat(self.rotation).to_cols_array_2d()
        glam::Mat4::from_translation(self.position).to_cols_array_2d()
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
