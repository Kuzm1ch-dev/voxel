use glam::IVec3;
use std::{array, task};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use wgpu::naga::Block;
use winit::event::ElementState;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::img_utils::RgbaImg;
use crate::model::vertex::Vertex;
use crate::render::camera::{BoundingBox, Camera};
use crate::render::texture_manager::ChunkTextureAtlas;
use wgpu::{BindGroupLayout, BufferDescriptor, SamplerDescriptor, ShaderSource, TextureView};

use super::block::BlockType;
use super::block_registry::{self, BlockRegistry};

pub const CHUNK_SIZE_X: usize = 16;
pub const CHUNK_SIZE_Y: usize = 256;
pub const CHUNK_SIZE_Z: usize = 16;
const POOL_INITIAL_SIZE: usize = 64;
const MAX_VERTICES_PER_CHUNK: usize = CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z * 24; // 24 vertices per block worst case
const MAX_INDICES_PER_CHUNK: usize = CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z * 36; // 36 indices per block worst case

#[derive(Debug, Clone, Copy, EnumIter)]
enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

#[derive(Debug)]
struct AdjacentChunks<'a> {
    north: Option<&'a Chunk>,
    south: Option<&'a Chunk>,
    east: Option<&'a Chunk>,
    west: Option<&'a Chunk>,
    up: Option<&'a Chunk>,
    down: Option<&'a Chunk>,
}

impl<'a> AdjacentChunks<'a> {
    pub fn to_vec(&self) -> Vec<Option<&Chunk>> {
        vec![
            self.north,
            self.south,
            self.east,
            self.west,
            self.up,
            self.down,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    position: IVec3, // Chunk position in world space
    blocks: Vec<Option<BlockType>>,
    needs_mesh_update: bool,
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        let blocks = vec![None; CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z];
        Self {
            position,
            blocks: blocks,
            needs_mesh_update: true,
        }
    }

    pub fn get_chunk_position(pos: IVec3) -> IVec3 {
        IVec3::new(
            pos.x.div_euclid(CHUNK_SIZE_X as i32),
            pos.y.div_euclid(CHUNK_SIZE_Y as i32),
            pos.z.div_euclid(CHUNK_SIZE_Z as i32),
        )
    }

    pub fn get_block_position(pos: IVec3) -> IVec3 {
        IVec3::new(
            pos.x.rem_euclid(CHUNK_SIZE_X as i32),
            pos.y.rem_euclid(CHUNK_SIZE_Y as i32),
            pos.z.rem_euclid(CHUNK_SIZE_Z as i32),
        )
    }

    
    pub fn get_index(x: usize, y: usize, z: usize) -> usize {
        x * (CHUNK_SIZE_Y * CHUNK_SIZE_Z) + y * CHUNK_SIZE_Z + z
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<&BlockType> {
        if x >= CHUNK_SIZE_X || y >= CHUNK_SIZE_Y || z >= CHUNK_SIZE_Z {
            return None;
        }
        let index = Self::get_index(x, y, z);
        self.blocks.get(index)?.as_ref()
    }

    pub fn get_blocks(&self) -> &Vec<Option<BlockType>> {
        &self.blocks
    }

    pub fn get_block_by_world_pos(&self, world_pos: IVec3) -> Option<&BlockType> {
        let x = world_pos.x.rem_euclid(CHUNK_SIZE_X as i32) as usize;
        let y = world_pos.y.rem_euclid(CHUNK_SIZE_Y as i32) as usize;
        let z = world_pos.z.rem_euclid(CHUNK_SIZE_Z as i32) as usize;
        self.get_block(x, y, z)
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Option<BlockType>) {
        if x < CHUNK_SIZE_X && y < CHUNK_SIZE_Y && z < CHUNK_SIZE_Z {
            let index = Self::get_index(x, y, z);
            self.blocks[index] = block;
            self.needs_mesh_update = true;
        }
    }

    fn add_block_faces(
        &self,
        x: usize,
        y: usize,
        z: usize,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u16>,
        adjacent_chunks: [Option<Arc<Mutex<Chunk>>>; 6],
        block_registry: &BlockRegistry,
        atlas: &ChunkTextureAtlas,
    ) {
        //let block_type = self.blocks[x][y][z].clone();
        let base_index = vertices.len() as u16;
        let world_x = self.position.x * CHUNK_SIZE_X as i32 + x as i32;
        let world_y = self.position.y * CHUNK_SIZE_Y as i32 + y as i32;
        let world_z = self.position.z * CHUNK_SIZE_Z as i32 + z as i32;
        let block = self.get_block(x, y, z).unwrap();
        // Convert to coordinates for should_render_face
        let check_pos = (x as i32, y as i32, z as i32);
        const FACES: [(Direction, [f32; 3], [(f32, f32, f32); 4], [(f32, f32); 4]); 6] = [
            // Direction,    Normal,         Vertex positions,                                  UV coords
            (
                Direction::Up,
                [0.0, 1.0, 0.0],
                [
                    (0.0, 1.0, 0.0),
                    (1.0, 1.0, 0.0),
                    (1.0, 1.0, 1.0),
                    (0.0, 1.0, 1.0),
                ],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
            (
                Direction::Down,
                [0.0, -1.0, 0.0],
                [
                    (0.0, 0.0, 1.0),
                    (1.0, 0.0, 1.0),
                    (1.0, 0.0, 0.0),
                    (0.0, 0.0, 0.0),
                ],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
            (
                Direction::South,
                [0.0, 0.0, 1.0],
                [
                    (0.0, 1.0, 1.0),
                    (1.0, 1.0, 1.0),
                    (1.0, 0.0, 1.0),
                    (0.0, 0.0, 1.0),
                ],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
            (
                Direction::North,
                [0.0, 0.0, -1.0],
                [
                    (1.0, 1.0, 0.0),
                    (0.0, 1.0, 0.0),
                    (0.0, 0.0, 0.0),
                    (1.0, 0.0, 0.0),
                ],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
            (
                Direction::East,
                [1.0, 0.0, 0.0],
                [
                    (1.0, 1.0, 1.0),
                    (1.0, 1.0, 0.0),
                    (1.0, 0.0, 0.0),
                    (1.0, 0.0, 1.0),
                ],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
            (
                Direction::West,
                [-1.0, 0.0, 0.0],
                [
                    (0.0, 1.0, 0.0), // Reorder west face
                    (0.0, 1.0, 1.0),
                    (0.0, 0.0, 1.0),
                    (0.0, 0.0, 0.0),
                ],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
        ];
        let arc_adjacent_chunks = Arc::new(adjacent_chunks);
        for (direction, normal, positions, uvs) in FACES {
            let (check_x, check_y, check_z) = match direction {
                Direction::Up => (check_pos.0, check_pos.1 + 1, check_pos.2),
                Direction::Down => (check_pos.0, check_pos.1 - 1, check_pos.2),
                Direction::South => (check_pos.0, check_pos.1, check_pos.2 + 1),
                Direction::North => (check_pos.0, check_pos.1, check_pos.2 - 1),
                Direction::East => (check_pos.0 + 1, check_pos.1, check_pos.2),
                Direction::West => (check_pos.0 - 1, check_pos.1, check_pos.2),
            };
            if self.should_render_face(check_x, check_y, check_z, arc_adjacent_chunks.clone(), direction) {
                let base_index = vertices.len() as u16;
                let face_texture = match direction {
                    Direction::Up => block.textures.top.clone(),
                    Direction::Down => block.textures.bottom.clone(),
                    Direction::South => block.textures.back.clone(),
                    Direction::North => block.textures.front.clone(),
                    Direction::East => block.textures.right.clone(),
                    Direction::West => block.textures.left.clone(),
                };
                let mut occulusion_vertex_map: HashMap<usize, f32> = HashMap::new();
                let occulusion_factor = 0.35;
                let occulusion_default = 1.0;
                match direction {
                    Direction::Up => {
                        if self.exist_block(x as i32, y as i32 +1, z as i32 - 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32, y as i32 +1, z as i32 + 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 - 1, y as i32 +1, z as i32, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 + 1, y as i32 +1, z as i32, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                    }
                    Direction::Down => {
                        if self.exist_block(x as i32, y as i32 -1, z as i32 - 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32, y as i32 -1, z as i32 + 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 - 1, y as i32 -1, z as i32, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 + 1, y as i32 -1, z as i32, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                    }
                    Direction::South => {
                        if self.exist_block(x as i32, y as i32, z as i32 + 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32, y as i32 -1, z as i32 + 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 - 1, y as i32, z as i32 + 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 + 1, y as i32, z as i32 + 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                    }
                    Direction::North => {
                        if self.exist_block(x as i32, y as i32, z as i32 - 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32, y as i32 -1, z as i32 - 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 - 1, y as i32, z as i32 - 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 + 1, y as i32, z as i32 - 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                    }
                    Direction::East => {
                        if self.exist_block(x as i32 + 1, y as i32, z as i32, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 + 1, y as i32 -1, z as i32, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 + 1, y as i32, z as i32 - 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 + 1, y as i32, z as i32 + 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                    }
                    Direction::West => {
                        if self.exist_block(x as i32 - 1, y as i32, z as i32, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 - 1, y as i32 -1, z as i32, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 - 1, y as i32, z as i32 - 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(0, occulusion_vertex_map.get(&0).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(3, occulusion_vertex_map.get(&3).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                        if self.exist_block(x as i32 - 1, y as i32, z as i32 + 1, arc_adjacent_chunks.clone()){
                            occulusion_vertex_map.insert(1, occulusion_vertex_map.get(&1).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                            occulusion_vertex_map.insert(2, occulusion_vertex_map.get(&2).unwrap_or(&occulusion_default).clone() - occulusion_factor);
                        }
                    }
                    _ => {
                        occulusion_vertex_map.insert(0, 1.0);
                        occulusion_vertex_map.insert(1, 1.0);
                        occulusion_vertex_map.insert(2, 1.0);
                        occulusion_vertex_map.insert(3, 1.0);
                    }
                }
                for i in 0..4 {
                    /*
                    0---1
                    |   |
                    3---2
                    */
                    let occulusion = occulusion_vertex_map.get(&i).unwrap_or(&1.0);
                    vertices.push(Vertex::new(
                        [
                            world_x as f32 + positions[i].0,
                            world_y as f32 + positions[i].1,
                            world_z as f32 + positions[i].2,
                        ],
                        normal,
                        [uvs[i].0, uvs[i].1],
                        atlas.get_texture_index(face_texture.as_str()).unwrap(),
                        occulusion.clone(),
                    ));
                }

                // Add indices for this face
                indices.extend_from_slice(&[
                    base_index,
                    base_index + 1,
                    base_index + 2,
                    base_index + 2,
                    base_index + 3,
                    base_index,
                ]);
            }
        }
    }

    fn exist_block(
        &self,
        x: i32,
        y: i32,
        z: i32,
        adjacent_chunks: Arc<[Option<Arc<Mutex<Chunk>>>; 6]>,
    ) -> bool {
        if x >= 0
            && x < CHUNK_SIZE_X as i32
            && y >= 0
            && y < CHUNK_SIZE_Y as i32
            && z >= 0
            && z < CHUNK_SIZE_Z as i32
        {
            return self.get_block(x as usize, y as usize, z as usize) != None;
        }
            if z < 0 {
                if let Some(chunk) = adjacent_chunks[0].clone() {
                    let chunk_lock = chunk.lock().unwrap();
                    return chunk_lock.get_block(x as usize, y as usize, CHUNK_SIZE_Z as usize - 1) != None;
                }
            }
            if z >= CHUNK_SIZE_Z as i32 {
                if let Some(chunk) = adjacent_chunks[1].clone() {
                    let chunk_lock = chunk.lock().unwrap();
                    return chunk_lock.get_block(x as usize, y as usize, 0) != None;
                }
            }
            if x < 0 {
                if let Some(chunk) = adjacent_chunks[2].clone() {
                    let chunk_lock = chunk.lock().unwrap();
                    return chunk_lock.get_block(CHUNK_SIZE_X as usize - 1, y as usize, z as usize) != None;
                }
            }
            if x >= CHUNK_SIZE_X as i32 {
                if let Some(chunk) = adjacent_chunks[3].clone() {
                    let chunk_lock = chunk.lock().unwrap();
                    return chunk_lock.get_block(0, y as usize, z as usize) != None;
                }
            }
            if y < 0 {
                if let Some(chunk) = adjacent_chunks[4].clone(){
                    let chunk_lock = chunk.lock().unwrap();
                    return chunk_lock.get_block(x as usize, CHUNK_SIZE_Y as usize - 1, z as usize) != None;
                }
            }
            if y >= CHUNK_SIZE_Y as i32 {
                if let Some(chunk) = adjacent_chunks[5].clone() {
                    let chunk_lock = chunk.lock().unwrap();
                    return chunk_lock.get_block(x as usize, 0, z as usize) != None;
                }
            }
            return false;
    }

    fn should_render_face(
        &self,
        x: i32,
        y: i32,
        z: i32,
        adjacent_chunks: Arc<[Option<Arc<Mutex<Chunk>>>; 6]>,
        direction: Direction,
    ) -> bool {
        // Check if the adjacent block is within the current chunk
        if x >= 0
            && x < CHUNK_SIZE_X as i32
            && y >= 0
            && y < CHUNK_SIZE_Y as i32
            && z >= 0
            && z < CHUNK_SIZE_Z as i32
        {
            return self.get_block(x as usize, y as usize, z as usize) == None;
        }
        let (chunk, new_x, new_y, new_z) = match direction{
            Direction::North => {
                if let Some(chunk) = adjacent_chunks[0].clone(){
                    (chunk, x as usize, y as usize, CHUNK_SIZE_Z as usize - 1)
                }else{
                    return true;
                }
            },
            Direction::South => {
                if let Some(chunk) = adjacent_chunks[1].clone(){
                    (chunk, x as usize, y as usize, 0)
                }else{
                    return true;
                }
            },
            Direction::East => {
                if let Some(chunk) = adjacent_chunks[2].clone(){
                    (chunk, 0, y as usize, z as usize)
                }else{
                    return true;
                }
            },
            Direction::West => {
                if let Some(chunk) = adjacent_chunks[3].clone(){
                    (chunk, CHUNK_SIZE_X as usize - 1, y as usize, z as usize)
                }else{
                    return true;
                }
            },
            Direction::Up => {
                if let Some(chunk) = adjacent_chunks[4].clone(){
                    (chunk, x as usize, 0, z as usize)
                }else{
                    return true;
                }
            },
            Direction::Down => {
                if let Some(chunk) = adjacent_chunks[5].clone(){
                    (chunk, x as usize, CHUNK_SIZE_Y as usize - 1, z as usize)
                }else{
                    return true;
                }
            },
            _ => return false,
        };
        let chunk_lock = chunk.lock().unwrap();
        chunk_lock.get_block(new_x, new_y, new_z) == None
    }
}

fn generate_mesh_data(
    chunk: &Chunk,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    adjacent_chunks: [Option<Arc<Mutex<Chunk>>>; 6],
    block_registry: &BlockRegistry,
    texture_atlas_bind_group_layout: &BindGroupLayout,
) -> (Vec<Vertex>, Vec<u16>, ChunkTextureAtlas) {
    let mut vertices = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let atlas = ChunkTextureAtlas::new(
        device,
        queue,
        &chunk,
        block_registry,
        texture_atlas_bind_group_layout,
    );
    for x in 0..CHUNK_SIZE_X {
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                if chunk.get_block(x, y, z) == None {
                    continue;
                }
                chunk.add_block_faces(
                    x,
                    y,
                    z,
                    &mut vertices,
                    &mut indices,
                    adjacent_chunks.clone(),
                    block_registry,
                    &atlas,
                );
            }
        }
    }
    (vertices, indices, atlas)
}

struct BufferPair {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

struct MeshPool {
    available_buffers: VecDeque<BufferPair>,
    device: Arc<wgpu::Device>,
}

impl MeshPool {
    fn new(device: Arc<wgpu::Device>) -> Self {
        let mut available_buffers = VecDeque::with_capacity(POOL_INITIAL_SIZE);

        // Pre-allocate buffers
        for _ in 0..POOL_INITIAL_SIZE {
            available_buffers.push_back(Self::create_buffer_pair(&device));
        }

        Self {
            available_buffers,
            device,
        }
    }

    fn create_buffer_pair(device: &wgpu::Device) -> BufferPair {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pooled Chunk Vertex Buffer"),
            size: (MAX_VERTICES_PER_CHUNK * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pooled Chunk Index Buffer"),
            size: (MAX_INDICES_PER_CHUNK * std::mem::size_of::<u16>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        BufferPair {
            vertex_buffer,
            index_buffer,
        }
    }

    fn acquire_buffers(&mut self) -> BufferPair {
        if let Some(buffers) = self.available_buffers.pop_front() {
            buffers
        } else {
            // Create new buffers if pool is empty
            Self::create_buffer_pair(&self.device)
        }
    }

    fn return_buffers(&mut self, buffers: BufferPair) {
        self.available_buffers.push_back(buffers);
    }
}

struct ChunkMeshManager {
    mesh_pool: MeshPool,
    active_meshes: HashMap<IVec3, (BufferPair, u32, ChunkTextureAtlas)>, // (buffers, index_count)
    queue: Arc<wgpu::Queue>,
}

impl ChunkMeshManager {
    fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            mesh_pool: MeshPool::new(device),
            active_meshes: HashMap::new(),
            queue,
        }
    }

    fn update_mesh(
        &mut self,
        chunk_pos: IVec3,
        vertices: &[Vertex],
        indices: &[u16],
        atlas: &ChunkTextureAtlas,
    ) {
        let buffers = if let Some((existing_buffers, _, _)) = self.active_meshes.remove(&chunk_pos)
        {
            existing_buffers
        } else {
            self.mesh_pool.acquire_buffers()
        };
        // Upload new mesh data
        self.queue
            .write_buffer(&buffers.vertex_buffer, 0, bytemuck::cast_slice(vertices));
        self.queue
            .write_buffer(&buffers.index_buffer, 0, bytemuck::cast_slice(indices));

        self.active_meshes
            .insert(chunk_pos, (buffers, indices.len() as u32, atlas.clone()));
    }

    fn remove_mesh(&mut self, chunk_pos: &IVec3) {
        if let Some((buffers, _, _)) = self.active_meshes.remove(chunk_pos) {
            self.mesh_pool.return_buffers(buffers);
        }
    }
}

struct MeshGenerationTask {
    chunk_pos: IVec3,
    chunk: Arc<Mutex<Chunk>>,
    neighbors: [Option<Arc<Mutex<Chunk>>>; 6],
}

struct ChunkMeshData {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    atlas: ChunkTextureAtlas,
}

struct MeshGenerationResult {
    chunk_pos: IVec3,
    mesh_data: ChunkMeshData, // You'll need to define this struct to hold vertex/index data
}

pub struct ChunkManager {
    chunks: HashMap<IVec3, Chunk>,
    mesh_manager: ChunkMeshManager,
    update_queue: VecDeque<IVec3>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    block_registry: Arc<Mutex<BlockRegistry>>,
    mesh_sender: Sender<MeshGenerationTask>,
    mesh_receiver: Receiver<MeshGenerationResult>,

}

impl ChunkManager {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        block_registry: Arc<Mutex<BlockRegistry>>,
    ) -> Self {
        let (task_sender, task_receiver) = channel::<MeshGenerationTask>(); // создаем канал
        let (result_sender, result_receiver) = channel::<MeshGenerationResult>();
        let device_clone = device.clone();
        let queue_clone = queue.clone();
        let block_registry_clone = block_registry.clone();
        // Spawn mesh generation thread
        let worker = thread::spawn(move || {
            while let Ok(task) = task_receiver.recv() {
                let chunk_lock = task.chunk.lock().unwrap();
                let adjacent_chunks = task.neighbors.clone();
                let block_registry_clone_lock = block_registry_clone.lock().unwrap();
                let texture_atlas_bind_group_layout =
                device_clone
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("Chunk Texture Bind Group Layout"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                    view_dimension: wgpu::TextureViewDimension::D2Array,
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
                let (vertices, indices, atlas) = generate_mesh_data(
                    &chunk_lock,
                    &device_clone,
                    &queue_clone,
                    adjacent_chunks,
                    &block_registry_clone_lock,
                    &texture_atlas_bind_group_layout,
                );
                let mesh_data = ChunkMeshData {
                    vertices,
                    indices,
                    atlas,
                };
                let result = MeshGenerationResult {
                    chunk_pos: task.chunk_pos,
                    mesh_data,
                };
                if result_sender.send(result).is_err() {
                    break;
                }
                println!("Get task on (re)generate chunk");
            }
        });
        Self {
            chunks: HashMap::new(),
            mesh_manager: ChunkMeshManager::new(device.clone(), queue.clone()),
            update_queue: VecDeque::new(),
            device,
            queue,
            block_registry,
            mesh_sender: task_sender,
            mesh_receiver: result_receiver
        }
    }

    pub fn get_chunk(&mut self, position: IVec3) -> Option<Mutex<&mut Chunk>> {
        if let Some(chunk) = self.chunks.get_mut(&position) {
            Some(Mutex::new(chunk))
        } else {
            None
        }
    }

    pub fn update_chunk_by_pos(&mut self, position: IVec3) {
        // Queue mesh updates
        self.update_queue.push_back(position);

        // Queue adjacent chunks for update
        for adj_pos in self.get_adjacent_chunk_positions(position) {
            if self.chunks.contains_key(&adj_pos) {
                self.update_queue.push_back(adj_pos);
            }
        }
    }

    pub fn update_chunk(&mut self, position: IVec3, blocks: Vec<Option<BlockType>>) {
        // Update chunk data
        if let Some(chunk) = self.chunks.get_mut(&position) {
            chunk.blocks = blocks;
            chunk.needs_mesh_update = true;
        } else {
            let mut chunk = Chunk::new(position);
            chunk.blocks = blocks;
            self.chunks.insert(position, chunk);
        }

        // Queue mesh updates
        self.update_queue.push_back(position);

        // Queue adjacent chunks for update
        for adj_pos in self.get_adjacent_chunk_positions(position) {
            if self.chunks.contains_key(&adj_pos) {
                self.update_queue.push_back(adj_pos);
            }
        }
    }

    pub fn process_mesh_updates(&mut self) {
        // Process a limited number of updates per frame
        const UPDATES_PER_FRAME: usize = 4;
        // let texture_atlas_bind_group_layout =
        //     self.device
        //         .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //             label: Some("Chunk Texture Bind Group Layout"),
        //             entries: &[
        //                 wgpu::BindGroupLayoutEntry {
        //                     binding: 0,
        //                     visibility: wgpu::ShaderStages::FRAGMENT,
        //                     ty: wgpu::BindingType::Texture {
        //                         sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                         view_dimension: wgpu::TextureViewDimension::D2Array,
        //                         multisampled: false,
        //                     },
        //                     count: None,
        //                 },
        //                 wgpu::BindGroupLayoutEntry {
        //                     binding: 1,
        //                     visibility: wgpu::ShaderStages::FRAGMENT,
        //                     ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //                     count: None,
        //                 },
        //             ],
        //         });
        while let Ok(result) = self.mesh_receiver.try_recv() {
            if result.mesh_data.vertices.is_empty() {
                self.mesh_manager.remove_mesh(&result.chunk_pos);
            } else {
                self.mesh_manager
                    .update_mesh(result.chunk_pos, &result.mesh_data.vertices, &result.mesh_data.indices, &result.mesh_data.atlas);
            }
        }
        for _ in 0..UPDATES_PER_FRAME {
            if let Some(chunk_pos) = self.update_queue.pop_front() {
                if let Some(chunk) = self.chunks.get(&chunk_pos) {
                    let adjacent_chunks = self.get_adjacent_chunks(chunk_pos);
                    let _ =  self.mesh_sender.send(MeshGenerationTask {
                        chunk_pos,
                        chunk: Arc::new(Mutex::new(chunk.clone())),
                        neighbors: [
                            adjacent_chunks.north.map(|c| Arc::new(Mutex::new(c.clone()))),
                            adjacent_chunks.south.map(|c| Arc::new(Mutex::new(c.clone()))),
                            adjacent_chunks.east.map(|c| Arc::new(Mutex::new(c.clone()))),
                            adjacent_chunks.west.map(|c| Arc::new(Mutex::new(c.clone()))),
                            adjacent_chunks.up.map(|c| Arc::new(Mutex::new(c.clone()))),
                            adjacent_chunks.down.map(|c| Arc::new(Mutex::new(c.clone()))),
                        ],
                    });
                    // let block_registry_lock = self.block_registry.lock().unwrap();
                    // let (vertices, indices, atlas) = generate_mesh_data(
                    //     chunk,
                    //     &self.device,
                    //     &self.queue,
                    //     Some(&adjacent_chunks),
                    //     &block_registry_lock,
                    //     &texture_atlas_bind_group_layout,
                    // );
                    // drop(block_registry_lock);
                    // if vertices.is_empty() {
                    //     self.mesh_manager.remove_mesh(&chunk_pos);
                    // } else {
                    //     self.mesh_manager
                    //         .update_mesh(chunk_pos, &vertices, &indices, &atlas);
                    // }
                }
            } else {
                break;
            }
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, camera: &mut Camera) {
        for (chunk_pos, (buffers, index_count, atlas)) in &self.mesh_manager.active_meshes {
            // Skip chunks outside view frustum
            // if !view_frustum.contains_chunk(*chunk_pos) {
            //     continue;
            // }
            let chunk_bbox = BoundingBox::from_chunk_position(*chunk_pos);
            if !camera.is_in_frustum(&chunk_bbox) {
                continue;
            }
            //render_pass.set_bind_group(4, &atlas.bind_group, &[]);
            render_pass.set_bind_group(3, &atlas.bind_group, &[]);
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..*index_count, 0, 0..1);
        }
    }

    fn unload_chunk(&mut self, position: &IVec3) {
        self.chunks.remove(position);
        self.mesh_manager.remove_mesh(position);
    }

    fn get_adjacent_chunks(&self, position: IVec3) -> AdjacentChunks {
        AdjacentChunks {
            north: self.chunks.get(&(position + IVec3::new(0, 0, -1))),
            south: self.chunks.get(&(position + IVec3::new(0, 0, 1))),
            east: self.chunks.get(&(position + IVec3::new(1, 0, 0))),
            west: self.chunks.get(&(position + IVec3::new(-1, 0, 0))),
            up: self.chunks.get(&(position + IVec3::new(0, 1, 0))),
            down: self.chunks.get(&(position + IVec3::new(0, -1, 0))),
        }
    }

    fn get_adjacent_chunk_positions(&self, position: IVec3) -> [IVec3; 6] {
        [
            position + IVec3::new(0, 0, -1), // North
            position + IVec3::new(0, 0, 1),  // South
            position + IVec3::new(1, 0, 0),  // East
            position + IVec3::new(-1, 0, 0), // West
            position + IVec3::new(0, 1, 0),  // Up
            position + IVec3::new(0, -1, 0), // Down
        ]
    }
}
