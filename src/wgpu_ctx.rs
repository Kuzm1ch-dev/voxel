use crate::img_utils::RgbaImg;
use crate::model::cube::Cube;
use crate::model::mesh::Mesh;
use crate::model::vertex::{self, create_box, create_vertex_buffer_layout};
use core::time;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use wgpu::hal::MAX_VERTEX_BUFFERS;
use wgpu::util::{BufferInitDescriptor, DeviceExt, RenderEncoder};
use wgpu::MemoryHints::Performance;
use wgpu::{BufferDescriptor, SamplerDescriptor, ShaderSource, TextureView};
use winit::event::WindowEvent::KeyboardInput;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

// In your Rust code
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

// Add a camera struct
struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn new(width: u32, height: u32) -> Self {
        Self {
            eye: glam::Vec3::new(3.0, 3.0, 3.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            aspect: width as f32 / height as f32,
            fovy: 45.0 * std::f32::consts::PI / 180.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}

pub struct WgpuCtx<'window> {
    start_time: Instant,
    running_time: f32,
    surface: wgpu::Surface<'window>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_index_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_image: RgbaImg,
    texture_size: wgpu::Extent3d,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: TextureView,
    camera: Camera,
    camera_controller: CameraController,
}

struct CameraController {
    rotation_x: f32,
    rotation_y: f32,
    rotation_z: f32,
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            rotation_x: 0.,
            rotation_y: 0.,
            rotation_z: 0.,
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let is_pressed = state == ElementState::Pressed;
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    fn update_camera(&self, camera: &mut Camera, dt: f32) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let right = forward_norm.cross(camera.up);

        if self.is_forward_pressed {
            camera.eye += forward_norm * self.speed * dt;
            camera.target += forward_norm * self.speed * dt;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed * dt;
            camera.target -= forward_norm * self.speed * dt;
        }
        if self.is_right_pressed {
            camera.eye += right * self.speed * dt;
            camera.target += right * self.speed * dt;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed * dt;
            camera.target -= right * self.speed * dt;
        }
        if self.is_up_pressed {
            camera.eye += camera.up * self.speed * dt;
            camera.target += camera.up * self.speed * dt;
        }
        if self.is_down_pressed {
            camera.eye -= camera.up * self.speed * dt;
            camera.target -= camera.up * self.speed * dt;
        }
    }
}

impl<'window> WgpuCtx<'window> {
    pub async fn new_async(window: Arc<Window>) -> WgpuCtx<'window> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let mut size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &surface_config);

        //let cube = Cube::new();
        //let vertex_list = cube.vertices();
        //let vertex_index_list = cube.indices();
        //let (vertex_list, vertex_index_list) = create_box();

        //let bytes: &[u8] = bytemuck::cast_slice(&vertex_list);
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 1024,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        //let vertex_index_bytes = bytemuck::cast_slice(&vertex_index_list);
        let vertex_index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Index Buffer"),
            size: 1024,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let img = RgbaImg::new("./assets/block.png").unwrap();
        let texture_size = wgpu::Extent3d {
            width: img.width,
            height: img.height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });
        //
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        //
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            create_pipeline(&device, surface_config.format, &render_pipeline_layout);

        let start_time = Instant::now();
        let running_time = 0.0;

        let camera = Camera::new(surface_config.width, surface_config.height);
        let camera_controller = CameraController::new(4.0);
        let depth_texture = Self::create_depth_texture(&device, &surface_config);

        WgpuCtx {
            start_time,
            running_time,
            surface,
            surface_config,
            adapter,
            device,
            queue,
            render_pipeline,
            vertex_buffer,
            vertex_index_buffer,
            texture,
            texture_image: img,
            texture_size,
            sampler,
            bind_group,
            uniform_bind_group,
            uniform_buffer,
            depth_texture,
            camera,
            camera_controller,
        }
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn new(window: Arc<Window>) -> WgpuCtx<'window> {
        pollster::block_on(WgpuCtx::new_async(window))
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            // Recreate depth texture on resize
            self.depth_texture = Self::create_depth_texture(&self.device, &self.surface_config);
        }
    }

    pub fn draw(&mut self) {
        let delta_time = self.start_time.elapsed().as_secs_f32() - self.running_time;
        self.running_time += delta_time;
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });


        // todo: Получаем данные чанков из мира
        let cube = Cube::new();
        let vertex_list = cube.vertices();
        let vertex_index_list = cube.indices();
        
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_bind_group(1, &self.uniform_bind_group, &[]);
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            rpass.set_index_buffer(
                self.vertex_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.draw_indexed(0..vertex_index_list.len() as u32, 0, 0..1);
            //rpass.draw(0..vertex_list.len() as u32, 0..1);
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.texture_image.bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.texture_image.width),
                rows_per_image: Some(self.texture_image.height),
            },
            self.texture_size,
        );
        self.camera_controller
            .update_camera(&mut self.camera, delta_time);
        let model = glam::Mat4::from_rotation_x(self.camera_controller.rotation_x)
            * glam::Mat4::from_rotation_y(self.camera_controller.rotation_y)
            * glam::Mat4::from_rotation_z(self.camera_controller.rotation_z);
        let view_proj = self.camera.build_view_projection_matrix();
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: model.to_cols_array_2d(),
        };
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertex_list));
        self.queue
            .write_buffer(&self.vertex_index_buffer, 0, bytemuck::cast_slice(&vertex_index_list));
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    pub fn keyboard_input(
        &mut self,
        device_id: DeviceId,
        event: KeyEvent,
        is_synthetic: bool,
    ) -> bool {
        match event.physical_key {
            PhysicalKey::Code(code) => self.camera_controller.process_keyboard(code, event.state),
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, dx: f32, dy: f32) {
        const SENSITIVITY: f32 = 0.005;

        let forward = (self.camera.target - self.camera.eye).normalize();
        let right = forward.cross(self.camera.up).normalize();

        // Rotate around right vector for up/down look
        let pitch_rotation = glam::Quat::from_axis_angle(right, -dy * SENSITIVITY);

        // Rotate around world up vector for left/right look
        let yaw_rotation = glam::Quat::from_axis_angle(glam::Vec3::Y, -dx * SENSITIVITY);

        let rotation = yaw_rotation * pitch_rotation;
        let forward = rotation * forward;

        self.camera.target = self.camera.eye + forward;
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[create_vertex_buffer_layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(swap_chain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,    // or Cw depending on your winding
            cull_mode: Some(wgpu::Face::Back),   // or None to disable culling for debugging
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
