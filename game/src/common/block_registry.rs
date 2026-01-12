use std::collections::HashMap;
use crate::common::block::Block;
use crate::blocks::{air::AirBlock, stone::StoneBlock, dirt::DirtBlock, grass::GrassBlock};



pub struct BlockRegistry {
    blocks: HashMap<String, Box<dyn Block>>,
    texture_paths: Vec<String>,
    texture_indices: HashMap<String, u32>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            blocks: HashMap::new(),
            texture_paths: Vec::new(),
            texture_indices: HashMap::new(),
        };
        
        registry.register_block(Box::new(AirBlock));
        registry.register_block(Box::new(StoneBlock));
        registry.register_block(Box::new(DirtBlock));
        registry.register_block(Box::new(GrassBlock));
        
        registry
    }
    
    fn register_block(&mut self, block: Box<dyn Block>) {
        let id = block.get_id();
        let texture_path = block.get_texture_path();
        
        println!("Registering block {} with texture: {}", id, texture_path);
        
        if !texture_path.is_empty() && !self.texture_paths.contains(&texture_path.to_string()) {
            let texture_index = self.texture_paths.len() as u32;
            self.texture_paths.push(texture_path.to_string());
            self.texture_indices.insert(id.to_string(), texture_index);
            println!("Block {} assigned texture index {}", id, texture_index);
        }
        
        self.blocks.insert(id.to_string(), block);
    }
    
    pub fn get_block(&self, id: &str) -> Option<&dyn Block> {
        self.blocks.get(id).map(|b| b.as_ref())
    }
    
    pub fn create_block(&self, id: &str) -> Option<Box<dyn Block>> {
        match id {
            "air" => Some(Box::new(AirBlock)),
            "stone" => Some(Box::new(StoneBlock)),
            "dirt" => Some(Box::new(DirtBlock)),
            "grass" => Some(Box::new(GrassBlock)),
            _ => None,
        }
    }
    
    pub fn get_texture_index(&self, block_id: &str) -> u32 {
        self.texture_indices.get(block_id).copied().unwrap_or(0)
    }
    
    pub fn get_texture_paths(&self) -> &Vec<String> {
        &self.texture_paths
    }
}

