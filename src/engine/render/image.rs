use glam::{Vec2, Vec4};
use std::collections::HashMap;

pub struct ImageRenderer {
    sampler: wgpu::Sampler,
    textures: HashMap<u32, (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup)>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ImageVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl ImageRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            sampler,
            textures: HashMap::new(),
        }
    }

    pub fn create_debug_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> (wgpu::Texture, wgpu::TextureView) {
        // Создаем простую отладочную текстуру 64x64 с шахматным узором
        let size = 64u32;
        let mut data = Vec::new();
        
        for y in 0..size {
            for x in 0..size {
                let checker = ((x / 8) + (y / 8)) % 2;
                let color = if checker == 0 { [255, 0, 255, 255] } else { [0, 255, 255, 255] };
                data.extend_from_slice(&color);
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Debug Texture"),
            size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
    
    pub fn load_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture_id: u32, rgba_data: &[u8], dimensions: (u32, u32)) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("UI Texture {}", texture_id)),
            size: wgpu::Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            wgpu::Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
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
            label: Some("UI Texture Bind Group Layout"),
        });
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some(&format!("UI Texture Bind Group {}", texture_id)),
        });
        
        self.textures.insert(texture_id, (texture, view, bind_group));
    }

}