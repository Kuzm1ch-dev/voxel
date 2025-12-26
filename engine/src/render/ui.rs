use wgpu::util::DeviceExt;
use glam::{Vec2, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl UIVertex {
    pub fn new(position: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self { position, uv, color }
    }
    
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct UIRenderer {
    render_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    vertices: Vec<UIVertex>,
    indices: Vec<u16>,
    texture_vertices: Vec<UIVertex>,
    texture_indices: Vec<u16>,
    texture_vertex_buffer: Option<wgpu::Buffer>,
    texture_index_buffer: Option<wgpu::Buffer>,
    current_texture_bind_group: Option<wgpu::BindGroup>,
}

impl UIRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[UIVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("UI Texture Bind Group Layout"),
        });

        let texture_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Texture Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui_texture.wgsl").into()),
        });

        let texture_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Texture Pipeline"),
            layout: Some(&texture_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &texture_shader,
                entry_point: Some("vs_main"),
                buffers: &[UIVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &texture_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            texture_pipeline,
            texture_bind_group_layout,
            vertex_buffer: None,
            index_buffer: None,
            vertices: Vec::new(),
            indices: Vec::new(),
            texture_vertices: Vec::new(),
            texture_indices: Vec::new(),
            texture_vertex_buffer: None,
            texture_index_buffer: None,
            current_texture_bind_group: None,
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.texture_vertices.clear();
        self.texture_indices.clear();
        self.current_texture_bind_group = None;
    }

    pub fn add_textured_rect(&mut self, pos: Vec2, size: Vec2, _texture_view: &wgpu::TextureView, bind_group: &wgpu::BindGroup) {
        let base_index = self.texture_vertices.len() as u16;
        
        self.texture_vertices.push(UIVertex::new([pos.x, pos.y], [0.0, 0.0], [1.0, 1.0, 1.0, 1.0]));
        self.texture_vertices.push(UIVertex::new([pos.x + size.x, pos.y], [1.0, 0.0], [1.0, 1.0, 1.0, 1.0]));
        self.texture_vertices.push(UIVertex::new([pos.x + size.x, pos.y + size.y], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]));
        self.texture_vertices.push(UIVertex::new([pos.x, pos.y + size.y], [0.0, 1.0], [1.0, 1.0, 1.0, 1.0]));

        self.texture_indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index + 2, base_index + 3, base_index,
        ]);
        
        // Store the bind group for rendering
        self.current_texture_bind_group = Some(bind_group.clone());
    }
    
    pub fn add_rect(&mut self, pos: Vec2, size: Vec2, color: Vec4) {
        let base_index = self.vertices.len() as u16;
        
        self.vertices.push(UIVertex::new([pos.x, pos.y], [0.0, 0.0], color.to_array()));
        self.vertices.push(UIVertex::new([pos.x + size.x, pos.y], [1.0, 0.0], color.to_array()));
        self.vertices.push(UIVertex::new([pos.x + size.x, pos.y + size.y], [1.0, 1.0], color.to_array()));
        self.vertices.push(UIVertex::new([pos.x, pos.y + size.y], [0.0, 1.0], color.to_array()));

        self.indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index + 2, base_index + 3, base_index,
        ]);
    }

    pub fn add_text(&mut self, text: &str, pos: Vec2, color: Vec4) {
        // Простой текст - один прямоугольник на всю строку
        let text_width = text.len() as f32 * 0.01;
        let text_height = 0.03;
        self.add_rect(pos, Vec2::new(text_width, text_height), color);
    }

    pub fn update_buffers(&mut self, device: &wgpu::Device) {
        if !self.vertices.is_empty() {
            self.vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }));

            self.index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }));
        }
        
        if !self.texture_vertices.is_empty() {
            self.texture_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Texture Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.texture_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }));

            self.texture_index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Texture Index Buffer"),
                contents: bytemuck::cast_slice(&self.texture_indices),
                usage: wgpu::BufferUsages::INDEX,
            }));
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Render regular colored UI elements
        if let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer) {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
        }
        
        // Render textured UI elements
        if let (Some(texture_vertex_buffer), Some(texture_index_buffer), Some(bind_group)) = 
            (&self.texture_vertex_buffer, &self.texture_index_buffer, &self.current_texture_bind_group) {
            render_pass.set_pipeline(&self.texture_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, texture_vertex_buffer.slice(..));
            render_pass.set_index_buffer(texture_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.texture_indices.len() as u32, 0, 0..1);
        }
    }
}