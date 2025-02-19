use std::collections::HashMap;
use std::path::Path;

use crate::world::block_registry::BlockRegistry;
use crate::world::chunk::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

#[derive(Debug, Clone)]
pub struct ChunkTextureAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
    pub texture_map: Vec<String>, // Lookup table for block -> texture mapping
    pub texture_count: u32,
}

impl ChunkTextureAtlas {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        chunk: &Chunk,
        block_registry: &BlockRegistry,
        bind_group_layout: &wgpu::BindGroupLayout
    ) -> Self {
        // Collect unique textures used in this chunk
        let mut unique_textures = std::collections::HashSet::new();
        
        // Scan chunk for used block types and their textures
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    if let Some(block) = chunk.get_block(x, y, z) {
                        unique_textures.insert(block.textures.top.clone());
                        unique_textures.insert(block.textures.bottom.clone());
                        unique_textures.insert(block.textures.front.clone());
                        unique_textures.insert(block.textures.back.clone());
                        unique_textures.insert(block.textures.left.clone());
                        unique_textures.insert(block.textures.right.clone());
                    }
                }
            }
        }
        let texture_indices: Vec<String> = unique_textures.into_iter().collect();
        let texture_count = texture_indices.len() as u32;

        // Create texture array only for textures used in this chunk
        let texture_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Chunk Texture Array"),
            size: wgpu::Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: texture_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Load and copy each texture into the array
        for (i, texture_name) in texture_indices.iter().enumerate() {
            let texture_path = Path::new("assets/textures/blocks").join(format!("{}.png", texture_name));
            let img = image::open(texture_path).unwrap().to_rgba8();
            
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture_array,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &img,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * 16),
                    rows_per_image: Some(16),
                },
                wgpu::Extent3d {
                    width: 16,
                    height: 16,
                    depth_or_array_layers: 1,
                },
            );
        }

        // Create view and sampler
        let view = texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Chunk Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture: texture_array,
            view,
            sampler,
            bind_group,
            texture_map: texture_indices,
            texture_count,
        }
    }

    // Get the index of a texture in the array
    pub fn get_texture_index(&self, texture_name: &str) -> Option<u32> {
        self.texture_map.iter().position(|x| x == texture_name).map(|i| i as u32)
    }
}
pub struct TextureManager {
    textures: HashMap<String, (wgpu::TextureView, wgpu::Sampler, wgpu::BindGroup)>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl TextureManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
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

        Self {
            textures: HashMap::new(),
            texture_bind_group_layout,
        }
    }

    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        texture_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Skip if texture is already loaded
        if self.textures.contains_key(name) {
            return Ok(());
        }

        let img = image::open(texture_path)?.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
            &img.into_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Bind Group", name)),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        self.textures.insert(name.to_string(), (view, sampler, bind_group));
        Ok(())
    }

    pub fn get_bind_group(&self, texture_name: &str) -> Option<&wgpu::BindGroup> {
        self.textures.get(texture_name).map(|(_, _, bind_group)| bind_group)
    }

    pub fn get_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }
}
