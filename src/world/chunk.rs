use glam::IVec3;
use noise::{NoiseFn, Perlin};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::{array, task};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use wgpu::naga::Block;
use winit::event::ElementState;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::img_utils::RgbaImg;
use crate::model::vertex::Vertex;
use crate::render::camera::{BoundingBox, Camera};
use crate::render::profiler::Profiler;
use crate::render::texture_manager::ChunkTextureAtlas;
use wgpu::{BindGroupLayout, BufferDescriptor, SamplerDescriptor, ShaderSource, TextureView};

use super::block::{BlockTextures, BlockType};
use super::block_registry::{self, BlockRegistry};

pub const CHUNK_SIZE_X: usize = 16;
pub const CHUNK_SIZE_Y: usize = 256;
pub const CHUNK_SIZE_Z: usize = 16;

#[derive(Debug, Clone, Copy, EnumIter, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

#[derive(Debug)]
pub struct AdjacentChunks<'a> {
    pub north: Option<&'a Chunk>,
    pub south: Option<&'a Chunk>,
    pub east: Option<&'a Chunk>,
    pub west: Option<&'a Chunk>,
}

impl<'a> AdjacentChunks<'a> {
    pub fn new(
        north: Option<&'a Chunk>,
        south: Option<&'a Chunk>,
        east: Option<&'a Chunk>,
        west: Option<&'a Chunk>,
    ) -> Self {
        Self {
            north,
            south,
            east,
            west,
        }
    }

    pub fn to_vec(&self) -> Vec<Option<&Chunk>> {
        vec![
            self.north, self.south, self.east, self.west
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    position: IVec3, // Chunk position in world space
    pub blocks: Vec<Option<BlockType>>,
    pub needs_mesh_update: bool,
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

    pub fn get_position(&self) -> IVec3 {
        self.position
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

    pub fn add_block_faces(
        &self,
        x: usize,
        y: usize,
        z: usize,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u16>,
        adjacent_chunks: [Option<Arc<Mutex<Chunk>>>; 4],
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
            if self.should_render_face(
                check_x,
                check_y,
                check_z,
                arc_adjacent_chunks.clone(),
                direction,
            ) {
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
                occulusion_vertex_map.insert(0, 1.);
                occulusion_vertex_map.insert(1, 1.);
                occulusion_vertex_map.insert(2, 1.);
                occulusion_vertex_map.insert(3, 1.);
                let occulusion_factor = 0.35;
                let occulusion_default = 1.0;
                
                match direction {
                    Direction::Up => {
                        if self.exist_block(
                            x as i32,
                            y as i32 + 1,
                            z as i32 - 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32,
                            y as i32 + 1,
                            z as i32 + 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 - 1,
                            y as i32 + 1,
                            z as i32,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 + 1,
                            y as i32 + 1,
                            z as i32,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                    }
                    Direction::Down => {
                        if self.exist_block(
                            x as i32,
                            y as i32 - 1,
                            z as i32 - 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32,
                            y as i32 - 1,
                            z as i32 + 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 - 1,
                            y as i32 - 1,
                            z as i32,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 + 1,
                            y as i32 - 1,
                            z as i32,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                    }
                    Direction::South => {
                        if self.exist_block(
                            x as i32,
                            y as i32,
                            z as i32 + 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32,
                            y as i32 - 1,
                            z as i32 + 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 - 1,
                            y as i32,
                            z as i32 + 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 + 1,
                            y as i32,
                            z as i32 + 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                    }
                    Direction::North => {
                        if self.exist_block(
                            x as i32,
                            y as i32,
                            z as i32 - 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32,
                            y as i32 - 1,
                            z as i32 - 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 - 1,
                            y as i32,
                            z as i32 - 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 + 1,
                            y as i32,
                            z as i32 - 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                    }
                    Direction::East => {
                        if self.exist_block(
                            x as i32 + 1,
                            y as i32,
                            z as i32,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 + 1,
                            y as i32 - 1,
                            z as i32,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 + 1,
                            y as i32,
                            z as i32 - 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 + 1,
                            y as i32,
                            z as i32 + 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                    }
                    Direction::West => {
                        if self.exist_block(
                            x as i32 - 1,
                            y as i32,
                            z as i32,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 - 1,
                            y as i32 - 1,
                            z as i32,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 - 1,
                            y as i32,
                            z as i32 - 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                0,
                                occulusion_vertex_map
                                    .get(&0)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                3,
                                occulusion_vertex_map
                                    .get(&3)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                        }
                        if self.exist_block(
                            x as i32 - 1,
                            y as i32,
                            z as i32 + 1,
                            arc_adjacent_chunks.clone(),
                        ) {
                            occulusion_vertex_map.insert(
                                1,
                                occulusion_vertex_map
                                    .get(&1)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
                            occulusion_vertex_map.insert(
                                2,
                                occulusion_vertex_map
                                    .get(&2)
                                    .unwrap_or(&occulusion_default)
                                    .clone()
                                    - occulusion_factor,
                            );
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
        adjacent_chunks: Arc<[Option<Arc<Mutex<Chunk>>>; 4]>,
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
                return chunk_lock.get_block(x as usize, y as usize, CHUNK_SIZE_Z as usize - 1)
                    != None;
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
                return chunk_lock.get_block(CHUNK_SIZE_X as usize - 1, y as usize, z as usize)
                    != None;
            }
        }
        if x >= CHUNK_SIZE_X as i32 {
            if let Some(chunk) = adjacent_chunks[3].clone() {
                let chunk_lock = chunk.lock().unwrap();
                return chunk_lock.get_block(0, y as usize, z as usize) != None;
            }
        }
        return false;
    }

    pub fn should_render_face(
        &self,
        x: i32,
        y: i32,
        z: i32,
        adjacent_chunks: Arc<[Option<Arc<Mutex<Chunk>>>; 4]>,
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
        let (chunk, new_x, new_y, new_z) = match direction {
            Direction::North => {
                if let Some(chunk) = adjacent_chunks[0].clone() {
                    (chunk, x as usize, y as usize, CHUNK_SIZE_Z as usize - 1)
                } else {
                    return true;
                }
            }
            Direction::South => {
                if let Some(chunk) = adjacent_chunks[1].clone() {
                    (chunk, x as usize, y as usize, 0)
                } else {
                    return true;
                }
            }
            Direction::East => {
                if let Some(chunk) = adjacent_chunks[2].clone() {
                    (chunk, 0, y as usize, z as usize)
                } else {
                    return true;
                }
            }
            Direction::West => {
                if let Some(chunk) = adjacent_chunks[3].clone() {
                    (chunk, CHUNK_SIZE_X as usize - 1, y as usize, z as usize)
                } else {
                    return true;
                }
            }
            _ => return false,
        };
        let chunk_lock = chunk.lock().unwrap();
        chunk_lock.get_block(new_x, new_y, new_z) == None
    }
}
