use crate::OPENGL_TO_WGPU_MATRIX;

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Instance {
    pub fn to_matrix(&self) -> InstanceMatrix {
        println!(
            "{:?}, {:?}",
            OPENGL_TO_WGPU_MATRIX * cgmath::Matrix4::from_translation(self.position),
            self.position
        );
        InstanceMatrix {
            model: (OPENGL_TO_WGPU_MATRIX * cgmath::Matrix4::from_translation(self.position))
                .into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceMatrix {
    model: [[f32; 4]; 4],
}

impl InstanceMatrix {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
