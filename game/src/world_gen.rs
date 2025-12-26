use voxel_engine::Vertex;
use std::collections::HashMap;
use crate::blocks::{BlockRegistry, BlockType};

const CHUNK_SIZE: usize = 16;
const CHUNK_HEIGHT: usize = 64;

pub struct Chunk {
    pub blocks: [[[BlockType; CHUNK_SIZE]; CHUNK_HEIGHT]; CHUNK_SIZE],
    pub position: (i32, i32),
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Chunk {
    pub fn new(x: i32, z: i32, registry: &BlockRegistry) -> Self {
        let mut chunk = Self {
            blocks: [[[BlockType::AIR; CHUNK_SIZE]; CHUNK_HEIGHT]; CHUNK_SIZE],
            position: (x, z),
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        
        chunk.generate_terrain();
        chunk.generate_mesh(registry);
        chunk
    }
    
    fn generate_terrain(&mut self) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = 32 + ((x + z) % 8) as usize;
                
                for y in 0..height.min(CHUNK_HEIGHT) {
                    if y < height - 4 {
                        self.blocks[x][y][z] = BlockType::STONE;
                    } else if y < height - 1 {
                        self.blocks[x][y][z] = BlockType::DIRT;
                    } else {
                        self.blocks[x][y][z] = BlockType::GRASS;
                    }
                }
            }
        }
    }
    
    fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x < 0 || x >= CHUNK_SIZE as i32 || y < 0 || y >= CHUNK_HEIGHT as i32 || z < 0 || z >= CHUNK_SIZE as i32 {
            return BlockType::AIR;
        }
        self.blocks[x as usize][y as usize][z as usize]
    }
    
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: BlockType, registry: &BlockRegistry) {
        if x >= CHUNK_SIZE || y >= CHUNK_HEIGHT || z >= CHUNK_SIZE {
            return;
        }
        self.blocks[x][y][z] = block_type;
        self.generate_mesh(registry);
    }
    
    fn generate_mesh(&mut self, registry: &BlockRegistry) {
        self.vertices.clear();
        self.indices.clear();
        
        let chunk_world_x = self.position.0 * CHUNK_SIZE as i32;
        let chunk_world_z = self.position.1 * CHUNK_SIZE as i32;
        
        for x in 0..CHUNK_SIZE as i32 {
            for y in 0..CHUNK_HEIGHT as i32 {
                for z in 0..CHUNK_SIZE as i32 {
                    let block = self.get_block(x, y, z);
                    if block == BlockType::AIR {
                        continue;
                    }
                    
                    let world_x = chunk_world_x + x;
                    let world_y = y;
                    let world_z = chunk_world_z + z;
                    
                    let tex_index = registry.get_texture_index(block.0);
                    
                    // Check each face and add if exposed
                    if self.get_block(x, y, z + 1) == BlockType::AIR {
                        self.add_face([world_x as f32, world_y as f32, (world_z + 1) as f32], [0.0, 0.0, 1.0], tex_index);
                    }
                    if self.get_block(x, y, z - 1) == BlockType::AIR {
                        self.add_face([world_x as f32, world_y as f32, world_z as f32], [0.0, 0.0, -1.0], tex_index);
                    }
                    if self.get_block(x + 1, y, z) == BlockType::AIR {
                        self.add_face([(world_x + 1) as f32, world_y as f32, world_z as f32], [1.0, 0.0, 0.0], tex_index);
                    }
                    if self.get_block(x - 1, y, z) == BlockType::AIR {
                        self.add_face([world_x as f32, world_y as f32, world_z as f32], [-1.0, 0.0, 0.0], tex_index);
                    }
                    if self.get_block(x, y + 1, z) == BlockType::AIR {
                        self.add_face([world_x as f32, (world_y + 1) as f32, world_z as f32], [0.0, 1.0, 0.0], tex_index);
                    }
                    if self.get_block(x, y - 1, z) == BlockType::AIR {
                        self.add_face([world_x as f32, world_y as f32, world_z as f32], [0.0, -1.0, 0.0], tex_index);
                    }
                }
            }
        }
    }
    
    fn add_face(&mut self, position: [f32; 3], normal: [f32; 3], tex_index: u32) {
        let base_index = self.vertices.len() as u16;
        
        match normal {
            [0.0, 0.0, 1.0] => { // Front
                self.vertices.push(Vertex::new([position[0], position[1], position[2]], normal, [0.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0] + 1.0, position[1], position[2]], normal, [1.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0] + 1.0, position[1] + 1.0, position[2]], normal, [1.0, 0.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1] + 1.0, position[2]], normal, [0.0, 0.0], tex_index, 1.0));
            },
            [0.0, 0.0, -1.0] => { // Back
                self.vertices.push(Vertex::new([position[0] + 1.0, position[1], position[2]], normal, [0.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1], position[2]], normal, [1.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1] + 1.0, position[2]], normal, [1.0, 0.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0] + 1.0, position[1] + 1.0, position[2]], normal, [0.0, 0.0], tex_index, 1.0));
            },
            [1.0, 0.0, 0.0] => { // Right
                self.vertices.push(Vertex::new([position[0], position[1], position[2] + 1.0], normal, [0.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1], position[2]], normal, [1.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1] + 1.0, position[2]], normal, [1.0, 0.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1] + 1.0, position[2] + 1.0], normal, [0.0, 0.0], tex_index, 1.0));
            },
            [-1.0, 0.0, 0.0] => { // Left
                self.vertices.push(Vertex::new([position[0], position[1], position[2]], normal, [0.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1], position[2] + 1.0], normal, [1.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1] + 1.0, position[2] + 1.0], normal, [1.0, 0.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1] + 1.0, position[2]], normal, [0.0, 0.0], tex_index, 1.0));
            },
            [0.0, 1.0, 0.0] => { // Top
                self.vertices.push(Vertex::new([position[0], position[1], position[2] + 1.0], normal, [0.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0] + 1.0, position[1], position[2] + 1.0], normal, [1.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0] + 1.0, position[1], position[2]], normal, [1.0, 0.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1], position[2]], normal, [0.0, 0.0], tex_index, 1.0));
            },
            [0.0, -1.0, 0.0] => { // Bottom
                self.vertices.push(Vertex::new([position[0], position[1], position[2]], normal, [0.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0] + 1.0, position[1], position[2]], normal, [1.0, 1.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0] + 1.0, position[1], position[2] + 1.0], normal, [1.0, 0.0], tex_index, 1.0));
                self.vertices.push(Vertex::new([position[0], position[1], position[2] + 1.0], normal, [0.0, 0.0], tex_index, 1.0));
            },
            _ => return,
        }
        
        self.indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index + 2, base_index + 3, base_index,
        ]);
    }
}

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub registry: BlockRegistry,
}

impl World {
    pub fn new() -> Self {
        let registry = BlockRegistry::new();
        let mut world = Self {
            chunks: HashMap::new(),
            registry,
        };
        
        // Generate chunks around origin
        for x in -1..=1 {
            for z in -1..=1 {
                let chunk = Chunk::new(x, z, &world.registry);
                world.chunks.insert((x, z), chunk);
            }
        }
        
        world
    }
    
    pub fn get_chunks(&self) -> &HashMap<(i32, i32), Chunk> {
        &self.chunks
    }
    
    pub fn get_chunks_mut(&mut self) -> &mut HashMap<(i32, i32), Chunk> {
        &mut self.chunks
    }
    
    pub fn break_block(&mut self, world_pos: (i32, i32, i32)) -> bool {
        let (world_x, world_y, world_z) = world_pos;
        let chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = world_z.div_euclid(CHUNK_SIZE as i32);
        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_y as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;
        
        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
            if local_y < CHUNK_HEIGHT && chunk.blocks[local_x][local_y][local_z] != BlockType::AIR {
                chunk.set_block(local_x, local_y, local_z, BlockType::AIR, &self.registry);
                return true;
            }
        }
        false
    }
    
    pub fn place_block(&mut self, world_pos: (i32, i32, i32), block_type: BlockType) -> bool {
        let (world_x, world_y, world_z) = world_pos;
        let chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = world_z.div_euclid(CHUNK_SIZE as i32);
        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_y as usize;
        let local_z = world_z.rem_euclid(CHUNK_SIZE as i32) as usize;
        
        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
            if local_y < CHUNK_HEIGHT && chunk.blocks[local_x][local_y][local_z] == BlockType::AIR {
                chunk.set_block(local_x, local_y, local_z, block_type, &self.registry);
                return true;
            }
        }
        false
    }
}