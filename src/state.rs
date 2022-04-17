use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::vertex::Vertex;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    solid_render_pipeline: wgpu::RenderPipeline,
    corner_render_pipeline: wgpu::RenderPipeline,
    active_render_pipeline_index: u32,

    tri_vertex_buffer: wgpu::Buffer,
    penta_vertex_buffer: wgpu::Buffer,
    tri_index_buffer: wgpu::Buffer,
    penta_index_buffer: wgpu::Buffer,
    active_buffer_index: u32,

    data: Data,
}

pub struct Data {
    pub bg_color: wgpu::Color,

    pub num_tri_vertices: u32,
    pub num_penta_vertices: u32,
    pub num_tri_indices: u32,
    pub num_penta_indices: u32,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                power_preference: wgpu::PowerPreference::HighPerformance,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    label: None,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let solid_shader =
            device.create_shader_module(&wgpu::include_wgsl!("../shaders/solid.wgsl"));
        let corner_shader =
            device.create_shader_module(&wgpu::include_wgsl!("../shaders/corners.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let solid_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &solid_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &solid_shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
        let corner_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &corner_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &corner_shader,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let tri_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Vertex Buffer"),
            contents: bytemuck::cast_slice(crate::TRIANGLE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let penta_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pentagon Vertex Buffer"),
            contents: bytemuck::cast_slice(crate::PENTAGON_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let tri_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Index Buffer"),
            contents: bytemuck::cast_slice(crate::TRIANGLE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let penta_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pentagon Index Buffer"),
            contents: bytemuck::cast_slice(crate::PENTAGON_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,

            solid_render_pipeline,
            corner_render_pipeline,
            active_render_pipeline_index: 0,

            tri_vertex_buffer,
            penta_vertex_buffer,
            tri_index_buffer,
            penta_index_buffer,
            active_buffer_index: 0,

            data: Data {
                bg_color: wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                },

                num_tri_vertices: crate::TRIANGLE_VERTICES.len() as u32,
                num_penta_vertices: crate::PENTAGON_VERTICES.len() as u32,
                num_tri_indices: crate::TRIANGLE_INDICES.len() as u32,
                num_penta_indices: crate::PENTAGON_INDICES.len() as u32,
            },
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.data.bg_color.r = position.x / self.size.width as f64;
                self.data.bg_color.g = position.y / self.size.height as f64;
                // println!("{:?}", position);
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::R),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.active_render_pipeline_index = 1 - self.active_render_pipeline_index;
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::S),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.active_buffer_index = 1 - self.active_buffer_index;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Extra block required because begin_render_pass takes &mut self
        // encoder.finish is only callable after the borrow is released
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.data.bg_color),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(match self.active_render_pipeline_index {
                0 => &self.solid_render_pipeline,
                1 => &self.corner_render_pipeline,
                _ => unreachable!(),
            });

            if self.active_buffer_index == 0 {
                render_pass.set_vertex_buffer(0, self.tri_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.tri_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.data.num_tri_indices, 0, 0..1);
            } else {
                render_pass.set_vertex_buffer(0, self.penta_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.penta_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.data.num_penta_indices, 0, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
