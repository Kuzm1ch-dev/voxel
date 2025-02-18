use glam::{IVec3, Vec3};
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::event::{DeviceId, ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::img_utils::RgbaImg;
use crate::model::vertex::Vertex;
use wgpu::{BufferDescriptor, SamplerDescriptor, ShaderSource, TextureFormat, TextureView};

use super::chunk::{create_initial_chunks, ChunkManager};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightViewProj {
    view_proj: [[f32; 4]; 4],
}

#[repr(C, align(16))] // Explicitly align to 16 bytes
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    direction: [f32; 4],    // 16 bytes (using vec4 alignment)
    color: [f32; 4],       // 16 bytes (using vec4 alignment)
    params: [f32; 4],      // 16 bytes for intensity, ambient_strength, and padding
}

impl LightUniform {
    fn new(direction: [f32; 3], color: [f32; 3], intensity: f32, ambient_strength: f32) -> Self {
        Self {
            direction: [direction[0], direction[1], direction[2], 0.0],
            color: [color[0], color[1], color[2], 0.0],
            params: [intensity, ambient_strength, 0.0, 0.0],
        }
    }
}
pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_points(points: &[Vec3]) -> Self {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for point in points {
            min = min.min(*point);
            max = max.max(*point);
        }

        Self { min, max }
    }

    pub fn from_chunk_position(chunk_position: IVec3) -> Self {
        let min = Vec3::new(
            chunk_position.x as f32 * 16.0,
            chunk_position.y as f32 * 16.0,
            chunk_position.z as f32 * 16.0,
        );
        let max = min + Vec3::new(16.0, 16.0, 16.0);

        Self { min, max }
    }
}

#[derive(Debug)]
pub struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            eye: glam::Vec3::new(3.0, 30.0, 3.0),
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

    fn calculate_frustum_planes(&self) -> [glam::Vec4; 6] {
        let view_proj = self.build_view_projection_matrix();
        let mut planes = [glam::Vec4::ZERO; 6];

        // Left plane
        planes[0] = view_proj.row(3) + view_proj.row(0);

        // Right plane
        planes[1] = view_proj.row(3) - view_proj.row(0);

        // Bottom plane
        planes[2] = view_proj.row(3) + view_proj.row(1);

        // Top plane
        planes[3] = view_proj.row(3) - view_proj.row(1);

        // Near plane
        planes[4] = view_proj.row(3) + view_proj.row(2);

        // Far plane
        planes[5] = view_proj.row(3) - view_proj.row(2);

        planes
    }

    pub fn is_in_frustum(&self, bbox: &BoundingBox) -> bool {
        let frustum_planes = self.calculate_frustum_planes(); 
        for plane in frustum_planes {
            let p = Vec3::new(
                if plane.x > 0.0 { bbox.max.x } else { bbox.min.x },
                if plane.y > 0.0 { bbox.max.y } else { bbox.min.y },
                if plane.z > 0.0 { bbox.max.z } else { bbox.min.z },
            );
            if plane.dot(p.extend(1.0)) < 0.0 {
                return false;
            }
        }
        true
    }
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

// In your Rust code
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
}

impl CameraUniform {
    fn update_view_proj(
        &mut self,
        camera: &mut Camera,
        camera_controller: &CameraController,
        dt: f32,
    ) {
        camera_controller.update_camera(camera, dt);
        let model = glam::Mat4::from_rotation_x(camera_controller.rotation_x)
            * glam::Mat4::from_rotation_y(camera_controller.rotation_y)
            * glam::Mat4::from_rotation_z(camera_controller.rotation_z);
        let view_proj = camera.build_view_projection_matrix();
        self.view_proj = view_proj.to_cols_array_2d();
        self.model = model.to_cols_array_2d();
    }
}

// Usage in main renderer
pub struct Renderer<'window> {
    start_time: Instant,
    running_time: f32,
    surface: wgpu::Surface<'window>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,

    render_pipeline: wgpu::RenderPipeline,
    depth_texture: TextureView,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,

    shadow_texture: wgpu::Texture,
    shadow_view: wgpu::TextureView,
    shadow_sampler: wgpu::Sampler,
    shadow_bind_group: wgpu::BindGroup,
    shadow_pipeline: wgpu::RenderPipeline,
    light_projection: glam::Mat4,

    light_view: LightViewProj,
    light_view_buffer: wgpu::Buffer,
    light_view_bind_group: wgpu::BindGroup,

    chunk_manager: ChunkManager,
    window_size: winit::dpi::PhysicalSize<u32>,
}

impl<'window> Renderer<'window> {
    pub fn new(window: Arc<Window>) -> Renderer<'window> {
        pollster::block_on(Renderer::new_async(window))
    }

    pub async fn new_async(window: Arc<Window>) -> Self {
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
                    memory_hints: wgpu::MemoryHints::Performance,
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
        //Camera
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        //Light
        let light_uniform = LightUniform::new(
            [-1.0, -1.0, -1.0], // direction (will be normalized in shader)
            [1.0, 1.0, 1.0],    // white light
            1.0,                // intensity
            0.3,                // ambient strength
        );
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });
        //Shadow
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Texture"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        // Create light view-projection matrix
        let light_position = glam::Vec3::new(20.0, 20.0, 20.0);
        let light_target = glam::Vec3::ZERO;
        let light_up = glam::Vec3::Y;

        let light_view = glam::Mat4::look_at_rh(light_position, light_target, light_up);
        let light_projection = glam::Mat4::orthographic_rh(
            -50.0, 50.0,  // left, right
            -50.0, 50.0,  // bottom, top
            -50.0, 50.0,  // near, far
        );
        let light_view_proj = light_projection * light_view;
        let light_view_proj = LightViewProj {
            view_proj: light_view_proj.to_cols_array_2d(),
        };
        let light_view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light View Buffer"),
            contents: bytemuck::cast_slice(&[light_view_proj]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let light_view_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light View Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let light_view_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light View Bind Group"),
            layout: &light_view_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_view_buffer.as_entire_binding(),
            }],
        });

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

        // Create shadow bind group
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        //
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout, 
                    &light_bind_group_layout, 
                    &shadow_bind_group_layout, 
                    &light_view_bind_group_layout
                    ],
                push_constant_ranges: &[],
            });
        let shadow_render_pipeline_layot =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Render Pipeline Layout"),
                bind_group_layouts: &[&light_view_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            create_pipeline(&device, surface_config.format, &render_pipeline_layout);
        let shadow_pipeline = create_shadow_pipeline(&device, &shadow_render_pipeline_layot);

        let camera = Camera::new(surface_config.width, surface_config.height);
        let camera_controller = CameraController::new(16.0);
        let depth_texture = Self::create_depth_texture(&device, &surface_config, "Depth Texture");

        // Create Chunk Manager
        let arc_device = Arc::new(device);
        let arc_queue = Arc::new(queue);
        let mut chunk_manager = ChunkManager::new(arc_device.clone(), arc_queue.clone());
        create_initial_chunks(&mut chunk_manager);
        let model = glam::Mat4::from_rotation_x(camera_controller.rotation_x)
            * glam::Mat4::from_rotation_y(camera_controller.rotation_y)
            * glam::Mat4::from_rotation_z(camera_controller.rotation_z);
        let view_proj = camera.build_view_projection_matrix();
        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            model: model.to_cols_array_2d(),
        };

        let start_time = Instant::now();
        let running_time = 0.0;

        Self {
            start_time,
            running_time,
            surface,
            device: arc_device,
            queue: arc_queue,
            config: surface_config,
            render_pipeline,
            depth_texture,

            camera,
            camera_uniform,
            camera_controller,
            camera_buffer,
            camera_bind_group,

            light_uniform,
            light_buffer,
            light_bind_group,

            shadow_texture,
            shadow_view,
            shadow_sampler,
            shadow_bind_group,
            shadow_pipeline,
            light_projection,
            light_view: light_view_proj,
            light_view_buffer,
            light_view_bind_group,

            chunk_manager,
            window_size: size,
        }
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> TextureView {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Self::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn update(&mut self, dt: f32) {
        self.camera_uniform
            .update_view_proj(&mut self.camera, &self.camera_controller, dt);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Process chunk updates
        //self.chunk_manager.process_mesh_updates();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let delta_time = self.start_time.elapsed().as_secs_f32() - self.running_time;
        self.running_time += delta_time;
        self.update(delta_time);
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
        self.chunk_manager.process_mesh_updates();
        {
            let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow Pass"),
                color_attachments: &[], // No color attachments for shadow pass
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            shadow_pass.set_pipeline(&self.shadow_pipeline);
            shadow_pass.set_bind_group(0, &self.light_view_bind_group, &[]);
            self.chunk_manager.render(&mut shadow_pass, &mut self.camera);
        }
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
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
            // rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_bind_group(1, &self.light_bind_group, &[]);
            rpass.set_bind_group(2, &self.shadow_bind_group, &[]);
            rpass.set_bind_group(3, &self.light_view_bind_group, &[]);
            self.chunk_manager.render(&mut rpass, &mut self.camera);
            // rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // rpass.set_index_buffer(
            //     self.vertex_index_buffer.slice(..),
            //     wgpu::IndexFormat::Uint16,
            // );
            //rpass.draw_indexed(0..vertex_index_list.len() as u32, 0, 0..1);
            //rpass.draw(0..vertex_list.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
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
        label: Some("Render Pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(swap_chain_format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw, // or Cw depending on your winding
            cull_mode: None,                  // or None to disable culling for debugging
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

fn create_shadow_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shadow Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shadow.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        cache: None,
        label: Some("Shadow Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            // Add this
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[], // Empty slice for no color attachments
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2, // Reduce shadow acne
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
