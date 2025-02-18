use std::{collections::HashMap, path::Path, sync::Arc};

use crate::render::texture_manager::TextureManager;

use super::block::{BlockTextures, BlockType};

pub struct BlockRegistry {
    blocks: HashMap<String, BlockType>,
    texture_manager: TextureManager,
}

impl BlockRegistry {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            blocks: HashMap::new(),
            texture_manager: TextureManager::new(&device),
        }
    }

    pub fn register_block(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: &str,
        textures: BlockTextures,
        textures_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Load all textures for the block
        let faces = [
            &textures.top,
            &textures.bottom,
            &textures.front,
            &textures.back,
            &textures.left,
            &textures.right,
        ];

        for texture_name in faces.iter() {
            let texture_path = textures_path.join(format!("{}.png", texture_name));
            self.texture_manager.load_texture(device, queue, texture_name, &texture_path)?;
        }

        // Register the block type
        self.blocks.insert(
            id.to_string(),
            BlockType {
                id: id.to_string(),
                textures,
            },
        );

        Ok(())
    }

    pub fn get_block(&self, id: &str) -> Option<BlockType> {
        self.blocks.get(id).cloned()
    }

    pub fn get_texture_bind_group(&self, texture_name: &str) -> Option<&wgpu::BindGroup> {
        self.texture_manager.get_bind_group(texture_name)
    }
}
