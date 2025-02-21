use glam::{IVec3, Vec3};
use rand::Rng;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use wgpu::core::device::queue;
use wgpu::util::DeviceExt;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window};

use crate::img_utils::RgbaImg;
use crate::model::vertex::Vertex;
use crate::world::block::BlockTextures;
use crate::world::block_registry::BlockRegistry;
use crate::world::chunk::ChunkManager;
use crate::world::world::World;
use wgpu::{
    BufferDescriptor, CommandEncoder, InstanceFlags, SamplerDescriptor, ShaderSource,
    TextureFormat, TextureView,
};

use super::camera::{Camera, CameraController, CameraUniform};
use super::light::LightUniform;
use super::profiler::{self, ProfileScope, Profiler};

// Usage in main renderer
pub struct Renderer<'window> {
    start_time: Instant,
    running_time: f32,
    profiler: Profiler,
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
    world: World,
    window_size: winit::dpi::PhysicalSize<u32>,
    ssao_compute_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
}

impl<'window> Renderer<'window> {
    pub fn new(window: Arc<Window>) -> Renderer<'window> {
        pollster::block_on(Renderer::new_async(window))
    }

    pub async fn new_async(window: Arc<Window>) -> Self {
        window.set_cursor_visible(false);
        window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
            .expect("Failed to grab cursor");

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            flags: InstanceFlags::all(),
            ..Default::default()
        });
        let window_size = window.inner_size();
        let center = winit::dpi::PhysicalPosition::new(
            window_size.width as i32 / 2,
            window_size.height as i32 / 2,
        );
        window
            .set_cursor_position(center)
            .expect("Failed to set cursor position");
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
        let limits = wgpu::Limits {
            max_texture_dimension_1d: 2048, // *
            max_texture_dimension_2d: 2048, // *
            max_texture_dimension_3d: 256,  // *
            max_texture_array_layers: 256,
            max_bind_groups: 4,
            max_bindings_per_bind_group: 1000,
            max_dynamic_uniform_buffers_per_pipeline_layout: 8,
            max_dynamic_storage_buffers_per_pipeline_layout: 0, // +
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 0,   // * +
            max_storage_textures_per_shader_stage: 1,  // +
            max_uniform_buffers_per_shader_stage: 11,  // +
            max_uniform_buffer_binding_size: 16 << 10, // * (16 KiB)
            max_storage_buffer_binding_size: 0,        // * +
            max_vertex_buffers: 8,
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 255, // +
            min_subgroup_size: 0,
            max_subgroup_size: 0,
            max_push_constant_size: 0,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_inter_stage_shader_components: 31,
            max_color_attachments: 8,
            max_color_attachment_bytes_per_sample: 32,
            max_compute_workgroup_storage_size: 0,      // +
            max_compute_invocations_per_workgroup: 256, // +
            max_compute_workgroup_size_x: 16,           // +
            max_compute_workgroup_size_y: 16,           // +
            max_compute_workgroup_size_z: 1,            // +
            max_compute_workgroups_per_dimension: 256,  // +
            max_buffer_size: 256 << 20,                 // (256 MiB),
            max_non_sampler_bindings: 1_000_000,
        };
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");
        let arc_device = Arc::new(device);
        let arc_queue = Arc::new(queue);

        let mut size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&arc_device, &surface_config);
        //Camera
        let camera_buffer = arc_device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            //size: std::mem::size_of::<CameraUniform>() as u64,
            size: 144 as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group_layout =
            arc_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let camera_bind_group = arc_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        //Light
        let light_position = glam::Vec3::new(20.0, 20.0, 20.0);
        let light_target = glam::Vec3::ZERO;
        let light_up = glam::Vec3::Y;

        let light_view = glam::Mat4::look_at_rh(light_position, light_target, light_up);
        let light_projection = glam::Mat4::orthographic_rh(
            -50.0, 50.0, // left, right
            -50.0, 50.0, // bottom, top
            -50.0, 50.0, // near, far
        );
        let light_view_proj = light_projection * light_view;
        let light_uniform = LightUniform::new(
            [-1.0, -1.0, -1.0], // direction (will be normalized in shader)
            [1.0, 1.0, 1.0],    // white light
            0.6,                // intensity
            0.3,                // ambient strength
            light_view_proj.to_cols_array_2d(),
        );
        let light_buffer = arc_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let light_bind_group_layout =
            arc_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let light_bind_group = arc_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });
        //Shadow
        let shadow_texture = arc_device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Texture"),
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
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
        let shadow_sampler = arc_device.create_sampler(&wgpu::SamplerDescriptor {
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
        let shadow_bind_group_layout =
            arc_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let shadow_bind_group = arc_device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        //Texture
        let depth_texture =
            Self::create_depth_texture(&arc_device, &surface_config, "Depth Texture");

        let texture_bind_group_layout =
            arc_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Chunk Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
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
            });
        //SSAO
        let ssao_texture = arc_device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let ssao_view = ssao_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let ssao_compute_bind_group_layout =
            arc_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("SSAO Bind Group Layout"),
            });

        let ssao_compute_bind_group = arc_device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ssao_compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&depth_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&ssao_view),
                },
            ],
            label: Some("SSAO Bind Group"),
        });

        //
        let compute_pipeline_layout =
            arc_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&ssao_compute_bind_group_layout],
                push_constant_ranges: &[],
                label: Some("SSAO Pipeline Layout"),
            });
        let render_pipeline_layout =
            arc_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                    &shadow_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let shadow_render_pipeline_layot =
            arc_device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Render Pipeline Layout"),
                bind_group_layouts: &[&light_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            create_pipeline(&arc_device, surface_config.format, &render_pipeline_layout);
        let shadow_pipeline = create_shadow_pipeline(&arc_device, &shadow_render_pipeline_layot);
        let compute_pipeline = create_compute_pipeline(&arc_device, &compute_pipeline_layout);

        let camera = Camera::new(surface_config.width, surface_config.height);
        let camera_controller = CameraController::new(16.0);
        // Create Chunk Manager

        let mut world = World::new(arc_device.clone(), arc_queue.clone());
        world.register_blocks(&arc_device, &arc_queue);
        world.create_initial_chunks(1);
        let model: glam::Mat4 = glam::Mat4::from_rotation_x(camera_controller.rotation_x)
            * glam::Mat4::from_rotation_y(camera_controller.rotation_y)
            * glam::Mat4::from_rotation_z(camera_controller.rotation_z);
        let view_proj = camera.build_view_projection_matrix();
        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            model: model.to_cols_array_2d(),
            view_position: camera.eye.to_array(),
        };

        let start_time = Instant::now();
        let running_time = 0.0;
        let mut profiler = Profiler::new();
        //profiler.toggle();
        Self {
            start_time,
            running_time,
            profiler,
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

            world,
            window_size: size,
            ssao_compute_bind_group,
            compute_pipeline,
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
        //self.device.poll(wgpu::Maintain::Wait);
        self.profiler.start_frame();
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
        self.world.process_mesh_updates();
        self.shadow_render_pass(&mut encoder);
        //self.compute_render_pass(&mut encoder);
        self.main_render_pass(&mut encoder, &texture_view);
        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
        self.profiler.end_frame();
        Ok(())
    }

    pub fn compute_render_pass(&mut self, encoder: &mut CommandEncoder) {
        let _shadow_scope = ProfileScope::new("SSAO Pass", &mut self.profiler);
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("SSAO Compute Pass"),
            timestamp_writes: Default::default(),
        });
        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.ssao_compute_bind_group, &[]);
        cpass.dispatch_workgroups(
            (self.window_size.width / 16) as u32,
            (self.window_size.height / 16) as u32,
            1,
        );
    }

    pub fn shadow_render_pass(&mut self, encoder: &mut CommandEncoder) {
        let _shadow_scope = ProfileScope::new("Shadow Pass", &mut self.profiler);
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
        shadow_pass.set_bind_group(0, &self.light_bind_group, &[]);
        self.world.render(&mut shadow_pass, &mut self.camera);
    }

    pub fn main_render_pass(&mut self, encoder: &mut CommandEncoder, texture_view: &TextureView) {
        let _render_scope = ProfileScope::new("Render Pass", &mut self.profiler);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture_view,
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
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.camera_bind_group, &[]);
        rpass.set_bind_group(1, &self.light_bind_group, &[]);
        rpass.set_bind_group(2, &self.shadow_bind_group, &[]);
        self.world.render(&mut rpass, &mut self.camera);
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

    pub fn process_mouse_motion(&mut self, dx: f32, dy: f32) {
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

    pub fn process_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Left => {
                if state == ElementState::Pressed {
                    let direction = (self.camera.target - self.camera.eye).normalize();
                    println!("eye {}", self.camera.eye);
                    println!("target {}", self.camera.target);
                    println!("direction {}", direction);
                    if let Some((pos, block)) = self
                        .world
                        .ray_cast(self.camera.target, direction, 25.0)
                    {

                        println!("Raycast hit at {:?} block: {:?}", pos, block);
                    }
                    //  }else{
                    //     println!("Raycast missed");
                    //  }
                }
            }
            MouseButton::Right => if state == ElementState::Pressed {},
            _ => {}
        }
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
        source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("./shaders/shader.wgsl"))),
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
            cull_mode: Some(wgpu::Face::Front), // or None to disable culling for debugging
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
        source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/shadow.wgsl").into()),
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
            cull_mode: Some(wgpu::Face::Front),
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

fn create_compute_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
) -> wgpu::ComputePipeline {
    let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("SSAO Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/ssao_compute.wgsl").into()),
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        cache: None,
        compilation_options: Default::default(),
        label: Some("SSAO Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &compute_shader,
        entry_point: Some("main"),
    });

    compute_pipeline
}

fn create_noise_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
    const NOISE_DIM: u32 = 4;

    let mut rng = rand::thread_rng();
    let mut noise_data = vec![0u8; (NOISE_DIM * NOISE_DIM * 4) as usize];

    for i in 0..(NOISE_DIM * NOISE_DIM * 4) as usize {
        noise_data[i] = rng.gen();
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Noise Texture"),
        size: wgpu::Extent3d {
            width: NOISE_DIM,
            height: NOISE_DIM,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &noise_data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(NOISE_DIM * 4),
            rows_per_image: Some(NOISE_DIM),
        },
        wgpu::Extent3d {
            width: NOISE_DIM,
            height: NOISE_DIM,
            depth_or_array_layers: 1,
        },
    );

    texture
}
