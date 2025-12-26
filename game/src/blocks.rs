use std::collections::HashMap;

pub trait Block {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> &'static str;
    fn get_texture_path(&self) -> &'static str;
    fn is_solid(&self) -> bool;
    fn is_transparent(&self) -> bool;
}

#[derive(Clone)]
pub struct AirBlock;
impl Block for AirBlock {
    fn get_id(&self) -> u32 { 0 }
    fn get_name(&self) -> &'static str { "air" }
    fn get_texture_path(&self) -> &'static str { "" }
    fn is_solid(&self) -> bool { false }
    fn is_transparent(&self) -> bool { true }
}

#[derive(Clone)]
pub struct StoneBlock;
impl Block for StoneBlock {
    fn get_id(&self) -> u32 { 1 }
    fn get_name(&self) -> &'static str { "stone" }
    fn get_texture_path(&self) -> &'static str { "game/assets/textures/block/stone.png" }
    fn is_solid(&self) -> bool { true }
    fn is_transparent(&self) -> bool { false }
}

#[derive(Clone)]
pub struct DirtBlock;
impl Block for DirtBlock {
    fn get_id(&self) -> u32 { 2 }
    fn get_name(&self) -> &'static str { "dirt" }
    fn get_texture_path(&self) -> &'static str { "game/assets/textures/block/dirt.png" }
    fn is_solid(&self) -> bool { true }
    fn is_transparent(&self) -> bool { false }
}

#[derive(Clone)]
pub struct GrassBlock;
impl Block for GrassBlock {
    fn get_id(&self) -> u32 { 3 }
    fn get_name(&self) -> &'static str { "grass" }
    fn get_texture_path(&self) -> &'static str { "game/assets/textures/block/grass.png" }
    fn is_solid(&self) -> bool { true }
    fn is_transparent(&self) -> bool { false }
}

pub struct BlockRegistry {
    blocks: HashMap<u32, Box<dyn Block>>,
    texture_paths: Vec<String>,
    texture_indices: HashMap<u32, u32>,
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
            self.texture_indices.insert(id, texture_index);
            println!("Block {} assigned texture index {}", id, texture_index);
        }
        
        self.blocks.insert(id, block);
    }
    
    pub fn get_block(&self, id: u32) -> Option<&dyn Block> {
        self.blocks.get(&id).map(|b| b.as_ref())
    }
    
    pub fn get_texture_index(&self, block_id: u32) -> u32 {
        self.texture_indices.get(&block_id).copied().unwrap_or(0)
    }
    
    pub fn get_texture_paths(&self) -> &Vec<String> {
        &self.texture_paths
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockType(pub u32);

impl BlockType {
    pub const AIR: BlockType = BlockType(0);
    pub const STONE: BlockType = BlockType(1);
    pub const DIRT: BlockType = BlockType(2);
    pub const GRASS: BlockType = BlockType(3);
}