use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use crate::{
    camera::{Camera, CameraController},
    deg_to_rad,
    instance::Instance,
    texture,
    vertex::Vertex,
};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,

    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
    num_indices: u32,

    camera: Camera,
    pub camera_controller: CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    bg_color: wgpu::Color,
    last_time: std::time::Instant,
}

const NUM_INSTANCES_PER_ROW: u32 = 10;

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

        let diffuse_bytes = include_bytes!("../tree.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, Some("Texture image"))
                .unwrap();

        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        count: None,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    },
                ],
            });
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &diffuse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            up: glam::Vec3::Y,
            front: (0.0, 0.0, -1.0).into(),

            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            zfar: 100.0,
            znear: 0.1,
        };
        let camera_controller = CameraController::new(15.0, 0.1);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&camera.build_vp_matrix().to_cols_array_2d()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let instance_displacement = glam::vec3(
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
        );
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = glam::Vec3::new(x as f32, 0.0, z as f32) - instance_displacement;
                    let rotation =
                        glam::Quat::from_axis_angle(position.normalize(), deg_to_rad(45.0));

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();
        let instance_data = instances
            .iter()
            .map(Instance::to_matrix)
            .collect::<Vec<_>>();
        println!("{:?}", instance_data);
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let shader = device.create_shader_module(&wgpu::include_wgsl!("../shaders/solid.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline layout"),
                bind_group_layouts: &[&diffuse_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pentagon Vertex Buffer"),
            contents: bytemuck::cast_slice(crate::PENTAGON_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

            render_pipeline,

            diffuse_bind_group,
            diffuse_texture,

            vertex_buffer,
            index_buffer,
            num_vertices: crate::PENTAGON_VERTICES.len() as u32,
            num_indices: crate::PENTAGON_INDICES.len() as u32,

            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,

            instances,
            instance_buffer,

            bg_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.2,
                a: 1.0,
            },
            last_time: std::time::Instant::now(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                // self.bg_color.r = position.x / self.size.width as f64;
                // self.bg_color.g = position.y / self.size.height as f64;
                // println!("{:?}", position);
            }
            _ => {}
        }
        self.camera_controller.process_events(event);
    }

    pub fn update(&mut self) {
        let current_time = std::time::Instant::now();
        let delta_time = current_time.duration_since(self.last_time).as_secs_f32();
        self.last_time = current_time;
        self.camera_controller
            .update_camera(&mut self.camera, delta_time);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&self.camera.build_vp_matrix().to_cols_array_2d()),
        );
    }

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
                            load: wgpu::LoadOp::Clear(self.bg_color),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as u32);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
