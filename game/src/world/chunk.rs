use voxel_engine::Vertex;
use crate::common::{block::Block, block_registry::BlockRegistry};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;

pub struct Chunk {
    pub blocks: Vec<Vec<Vec<Option<Box<dyn Block>>>>>,
    pub position: (i32, i32),
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Chunk {
    pub fn new(x: i32, z: i32, registry: &BlockRegistry) -> Self {
        let mut blocks = Vec::with_capacity(CHUNK_SIZE);
        for _ in 0..CHUNK_SIZE {
            let mut y_vec = Vec::with_capacity(CHUNK_HEIGHT);
            for _ in 0..CHUNK_HEIGHT {
                let mut z_vec = Vec::with_capacity(CHUNK_SIZE);
                for _ in 0..CHUNK_SIZE {
                    z_vec.push(None);
                }
                y_vec.push(z_vec);
            }
            blocks.push(y_vec);
        }
        
        let mut chunk = Self {
            blocks,
            position: (x, z),
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        
        chunk.generate_terrain(registry);
        chunk.generate_mesh(registry);
        chunk
    }
    
    fn generate_terrain(&mut self, registry: &BlockRegistry) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = 32 + ((x + z) % 8) as usize;
                
                for y in 0..height.min(CHUNK_HEIGHT) {
                    let block_name = if y < height - 4 {
                        "stone"
                    } else if y < height - 1 {
                        "dirt"
                    } else {
                        "grass"
                    };
                    
                    if let Some(block) = registry.create_block(block_name) {
                        self.blocks[x][y][z] = Some(block);
                    }
                }
            }
        }
    }
    
    fn get_block_name(&self, x: i32, y: i32, z: i32) -> &str {
        if x < 0 || x >= CHUNK_SIZE as i32 || y < 0 || y >= CHUNK_HEIGHT as i32 || z < 0 || z >= CHUNK_SIZE as i32 {
            return "air";
        }
        
        match &self.blocks[x as usize][y as usize][z as usize] {
            Some(block) => block.get_name(),
            None => "air",
        }
    }
    
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_name: &str, registry: &BlockRegistry) {
        if x >= CHUNK_SIZE || y >= CHUNK_HEIGHT || z >= CHUNK_SIZE {
            return;
        }
        
        if block_name == "air" {
            self.blocks[x][y][z] = None;
        } else if let Some(block) = registry.create_block(block_name) {
            self.blocks[x][y][z] = Some(block);
        }
        
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
                    let block_name = self.get_block_name(x, y, z);
                    if block_name == "air" {
                        continue;
                    }
                    
                    let world_x = chunk_world_x + x;
                    let world_y = y;
                    let world_z = chunk_world_z + z;
                    
                    let tex_index = registry.get_texture_index(block_name);
                    
                    // Check each face and add if exposed
                    if self.get_block_name(x, y, z + 1) == "air" {
                        self.add_face([world_x as f32, world_y as f32, (world_z + 1) as f32], [0.0, 0.0, 1.0], tex_index);
                    }
                    if self.get_block_name(x, y, z - 1) == "air" {
                        self.add_face([world_x as f32, world_y as f32, world_z as f32], [0.0, 0.0, -1.0], tex_index);
                    }
                    if self.get_block_name(x + 1, y, z) == "air" {
                        self.add_face([(world_x + 1) as f32, world_y as f32, world_z as f32], [1.0, 0.0, 0.0], tex_index);
                    }
                    if self.get_block_name(x - 1, y, z) == "air" {
                        self.add_face([world_x as f32, world_y as f32, world_z as f32], [-1.0, 0.0, 0.0], tex_index);
                    }
                    if self.get_block_name(x, y + 1, z) == "air" {
                        self.add_face([world_x as f32, (world_y + 1) as f32, world_z as f32], [0.0, 1.0, 0.0], tex_index);
                    }
                    if self.get_block_name(x, y - 1, z) == "air" {
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