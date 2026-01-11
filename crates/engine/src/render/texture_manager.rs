use std::{path::Path, sync::Arc};
use wgpu::{Sampler, TextureView, util::DeviceExt};

pub struct TextureManager {
    pub texture_array: wgpu::Texture,
    capacity: u32,
    next_index: u32,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl TextureManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, capacity: u32) -> Self {
        let texture_size = 16u32;
        let next_index = 0;
        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture Array"),
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
                depth_or_array_layers: capacity,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
    
        
        Self {
            texture_array,
            capacity,
            next_index,
            device,
            queue
        }
    }

    pub fn get_texture_view(&self) -> TextureView {
        self.texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    }

    pub fn get_sampler(&self) -> Sampler{
        self.device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("Texture Sampler"),
                    address_mode_u: wgpu::AddressMode::Repeat,
                    address_mode_v: wgpu::AddressMode::Repeat,
                    address_mode_w: wgpu::AddressMode::Repeat,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                })
    }

    pub fn add_texture(&mut self, path: &String){
        println!("Loading texture {}", path);
        if let Ok(img) = image::open(path) {
            println!("Successfully loaded texture: {}", path);
            let rgba = img.to_rgba8();
            let dimensions = rgba.dimensions();
            println!("Texture dimensions: {}x{}", dimensions.0, dimensions.1);
            
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture_array,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: self.next_index },
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
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
        }else {
            println!("Failed to load texture: {}, using magenta fallback", path);
            // Fallback to magenta texture if loading fails
            let magenta = [255, 0, 255, 255];
            let mut texture_data = Vec::new();
            for _ in 0..(self.texture_array.size().width * self.texture_array.size().height) {
                texture_data.extend_from_slice(&magenta);
            }
            
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture_array,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: self.next_index },
                    aspect: wgpu::TextureAspect::All,
                },
                &texture_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.texture_array.size().width),
                    rows_per_image: Some(self.texture_array.size().width),
                },
                wgpu::Extent3d {
                    width: self.texture_array.size().width,
                    height: self.texture_array.size().height,
                    depth_or_array_layers: 1,
                },
            );
        }
        self.next_index+=1;
    }

}