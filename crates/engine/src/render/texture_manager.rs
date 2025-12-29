use std::path::Path;
use wgpu::util::DeviceExt;

pub struct TextureManager {
    pub texture_array: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, texture_paths: &[String]) -> Self {
        let texture_size = 16u32;
        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture Array"),
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
                depth_or_array_layers: texture_paths.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        // Load each texture
        for (i, path) in texture_paths.iter().enumerate() {
            println!("Loading texture {}: {}", i, path);
            if let Ok(img) = image::open(path) {
                println!("Successfully loaded texture: {}", path);
                let rgba = img.to_rgba8();
                let dimensions = rgba.dimensions();
                println!("Texture dimensions: {}x{}", dimensions.0, dimensions.1);
                
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture_array,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &rgba,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * dimensions.0),
                        rows_per_image: Some(dimensions.1),
                    },
                    wgpu::Extent3d {
                        width: dimensions.0,
                        height: dimensions.1,
                        depth_or_array_layers: 1,
                    },
                );
            } else {
                println!("Failed to load texture: {}, using magenta fallback", path);
                // Fallback to magenta texture if loading fails
                let magenta = [255, 0, 255, 255];
                let mut texture_data = Vec::new();
                for _ in 0..(texture_size * texture_size) {
                    texture_data.extend_from_slice(&magenta);
                }
                
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture_array,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &texture_data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * texture_size),
                        rows_per_image: Some(texture_size),
                    },
                    wgpu::Extent3d {
                        width: texture_size,
                        height: texture_size,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
        
        let texture_view = texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Self {
            texture_array,
            texture_view,
            sampler,
        }
    }
}