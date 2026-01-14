use voxel_engine::Engine;
use voxel_engine::Vertex;
use std::collections::HashMap;
use crate::common::block_registry::BlockRegistry;
use crate::world::chunk::CHUNK_HEIGHT;
use crate::world::chunk::CHUNK_SIZE;
use crate::world::chunk::Chunk;


pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub registry: BlockRegistry,
}

impl World {
    pub fn new(engine: &mut Engine) -> Self {
        let registry = BlockRegistry::new(engine);
        let mut world = Self {
            chunks: HashMap::new(),
            registry,
        };
        
        // Generate chunks around origin
        for x in -1..=1 {
            for z in -1..=1 {
                let chunk = Chunk::new(engine, x, z, &world.registry);
                world.chunks.insert((x, z), chunk);
            }
        }
        
        world
    }
    
    pub fn render(&mut self, engine: &mut Engine) {
        engine.renderer.clear_meshes();
        
        // Regenerate meshes for dirty chunks
        for chunk in self.get_chunks_mut().values_mut() {
            // Chunk regenerates mesh automatically in set_block
        }
        
        for chunk in self.get_chunks().values() {
            if !chunk.vertices.is_empty() {
                let vertex_data = bytemuck::cast_slice(&chunk.vertices);
                engine.renderer.add_mesh(vertex_data, &chunk.indices);
            }
        }
    }


    pub fn get_chunks(&self) -> &HashMap<(i32, i32), Chunk> {
        &self.chunks
    }
    
    pub fn get_chunks_mut(&mut self) -> &mut HashMap<(i32, i32), Chunk> {
        &mut self.chunks
    }
    
    pub fn break_block(&mut self, engine: &Engine, world_pos: (i32, i32, i32)) -> bool {
        let (world_x, world_y, world_z) = world_pos;
        let chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = world_z.div_euclid(CHUNK_SIZE as i32);
        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_y as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;
        
        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
            if local_y < CHUNK_HEIGHT && chunk.blocks[local_x][local_y][local_z].is_some() {
                chunk.set_block(engine, local_x, local_y, local_z, "air", &self.registry);
                return true;
            }
        }
        false
    }
    
    pub fn place_block(&mut self, engine: &Engine, world_pos: (i32, i32, i32), block_name: &str) -> bool {
        let (world_x, world_y, world_z) = world_pos;
        let chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = world_z.div_euclid(CHUNK_SIZE as i32);
        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_y as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;
        
        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
            if local_y < CHUNK_HEIGHT && chunk.blocks[local_x][local_y][local_z].is_none() {
                chunk.set_block(engine, local_x, local_y, local_z, block_name, &self.registry);
                return true;
            }
        }
        false
    }
    
    pub fn get_block_at(&self, world_pos: (i32, i32, i32)) -> &str {
        let (world_x, world_y, world_z) = world_pos;
        let chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = world_z.div_euclid(CHUNK_SIZE as i32);
        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_y as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;
        
        if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_z)) {
            if local_y < CHUNK_HEIGHT {
                return match &chunk.blocks[local_x][local_y][local_z] {
                    Some(block) => block.get_name(),
                    None => "air",
                };
            }
        }
        "air"
    }
}