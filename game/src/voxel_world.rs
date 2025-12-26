use voxel_engine::Engine;
use crate::world_gen::World;
use crate::blocks::BlockType;

pub struct VoxelWorld {
    pub world: World,
}

impl VoxelWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }
    
    pub fn render(&mut self, engine: &mut Engine) {
        engine.renderer.clear_meshes();
        
        // Regenerate meshes for dirty chunks
        for chunk in self.world.get_chunks_mut().values_mut() {
            // Chunk regenerates mesh automatically in set_block
        }
        
        for chunk in self.world.get_chunks().values() {
            if !chunk.vertices.is_empty() {
                let vertex_data = bytemuck::cast_slice(&chunk.vertices);
                engine.renderer.add_mesh(vertex_data, &chunk.indices);
            }
        }
    }
    
    pub fn break_block(&mut self, world_pos: (i32, i32, i32)) -> bool {
        self.world.break_block(world_pos)
    }
    
    pub fn place_block(&mut self, world_pos: (i32, i32, i32), block_type: BlockType) -> bool {
        self.world.place_block(world_pos, block_type)
    }
}