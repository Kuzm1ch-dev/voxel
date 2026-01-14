use std::{collections::HashMap, sync::Arc};
use wgpu::{Sampler, TextureView};

pub struct TextureInfo {
    pub id: u32,
    pub path: String,
    pub dimensions: (u32, u32),
    pub atlas_position: (u32, u32, u32), // x, y, z в атласе
    pub uvs: (f32, f32, f32, f32), // 
}

pub struct TextureManager {
    pub texture_array: wgpu::Texture,
    atlas_size: u32,
    max_layers: u32,
    next_position: (u32, u32, u32), // x, y, z
    current_row_height: u32,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    textures: HashMap<String, TextureInfo>, // path -> info
    texture_by_id: HashMap<u32, String>,    // id -> path
    next_id: u32,
}

impl TextureManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, atlas_size: u32) -> Self {
        let max_layers = 256;
        
        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: max_layers,
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
            atlas_size,
            max_layers,
            next_position: (0, 0, 0),
            current_row_height: 0,
            device,
            queue,
            textures: HashMap::new(),
            texture_by_id: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn get_atlas_size(&self) -> u32 {
        self.atlas_size
    }

    pub fn add_texture(&mut self, path: &str, name: Option<&str>) -> Option<u32> {
        let texture_name = name.unwrap_or_else(|| {
            std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        });
        
        // Return existing texture if already loaded
        if let Some(info) = self.textures.get(texture_name) {
            return Some(info.id);
        }
        
        let (texture_data, width, height) = if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let dimensions = rgba.dimensions();
            (rgba.into_raw(), dimensions.0, dimensions.1)
        } else {
            println!("Failed to load texture: {}, using magenta fallback", path);
            // Magenta fallback 16x16
            let mut data = Vec::new();
            for _ in 0..(16 * 16) {
                data.extend_from_slice(&[255, 0, 255, 255]);
            }
            (data, 16, 16)
        };
        
        // Find position for texture
        let (x, y, z) = self.find_position(width, height)?;
        
        self.write_texture_data(&texture_data, x, y, z, width, height);
        
        let u_min = x as f32 / self.atlas_size as f32;
        let v_min = y as f32 / self.atlas_size as f32;
        let u_max = (x + width) as f32 / self.atlas_size as f32;
        let v_max = (y + height) as f32 / self.atlas_size as f32;      

        let texture_info = TextureInfo {
            id: self.next_id,
            path: path.to_string(),
            dimensions: (width, height),
            atlas_position: (x, y, z),
            uvs: (u_min, v_min, u_max, v_max)
        };
        
        self.textures.insert(texture_name.to_string(), texture_info);
        self.texture_by_id.insert(self.next_id, texture_name.to_string());
        
        let id: u32 = self.next_id;
        self.next_id += 1;
        Some(id)
    }
    
    pub fn get_texture_id_by_name(&self, name: &str) -> Option<u32> {
        self.textures.get(name).map(|info| info.id)
    }
    
    fn find_position(&mut self, width: u32, height: u32) -> Option<(u32, u32, u32)> {
        let (mut x, mut y, mut z) = self.next_position;
        
        // Check if texture fits in current row
        if x + width > self.atlas_size {
            // Move to next row
            x = 0;
            y += self.current_row_height;
            self.current_row_height = 0;
        }
        
        // Check if texture fits in current layer
        if y + height > self.atlas_size {
            // Move to next layer
            x = 0;
            y = 0;
            z += 1;
            self.current_row_height = 0;
        }
        
        // Check if we have space
        if z >= self.max_layers {
            println!("Texture atlas full! Cannot add more textures.");
            return None;
        }
        
        // Update position for next texture
        self.next_position = (x + width, y, z);
        self.current_row_height = self.current_row_height.max(height);
        
        Some((x, y, z))
    }
    
    fn write_texture_data(&self, data: &[u8], x: u32, y: u32, z: u32, width: u32, height: u32) {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture_array,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
    
    pub fn get_texture_info(&self, name: &str) -> Option<&TextureInfo> {
        self.textures.get(name)
    }
    
    pub fn get_texture_info_by_id(&self, id: u32) -> Option<&TextureInfo> {
        self.texture_by_id.get(&id).and_then(|name| self.textures.get(name))
    }
    
    pub fn get_capacity(&self) -> u32 {
        self.max_layers * self.atlas_size * self.atlas_size
    }

    pub fn get_texture_view(&self) -> TextureView {
        self.texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    }

    pub fn get_sampler(&self) -> Sampler {
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

}