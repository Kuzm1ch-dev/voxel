use glam::IVec3;
use wgpu::naga::Block;
use std::array;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
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


#[derive(Debug)]
pub struct Chunk {
    position: IVec3, // Chunk position in world space
    blocks: Vec<Option<BlockType>>,
    needs_mesh_update: bool
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        let blocks = vec![
            None;
            CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z
        ];
        Self {
            position,
            blocks: blocks,
            needs_mesh_update: true,
        }
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
        adjacent_chunks: Option<&AdjacentChunks>,
        block_registry: &Arc<BlockRegistry>,
        atlas: &ChunkTextureAtlas
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
                    (0.0, 1.0, 0.0),    // Reorder west face
                    (0.0, 1.0, 1.0),
                    (0.0, 0.0, 1.0),
                    (0.0, 0.0, 0.0),
                ],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
        ];

        for (direction, normal, positions, uvs) in FACES {
            let (check_x, check_y, check_z) = match direction {
                Direction::Up    => (check_pos.0, check_pos.1 + 1, check_pos.2),
                Direction::Down  => (check_pos.0, check_pos.1 - 1, check_pos.2),
                Direction::South => (check_pos.0, check_pos.1, check_pos.2 + 1),
                Direction::North => (check_pos.0, check_pos.1, check_pos.2 - 1),
                Direction::East  => (check_pos.0 + 1, check_pos.1, check_pos.2),
                Direction::West  => (check_pos.0 - 1, check_pos.1, check_pos.2),
            };
    
            if self.should_render_face(check_x, check_y, check_z, adjacent_chunks, direction) {
                let base_index = vertices.len() as u16;
                let face_texture = match direction {
                    Direction::Up => block.textures.top.clone(),
                    Direction::Down => block.textures.bottom.clone(),
                    Direction::South => block.textures.back.clone(), 
                    Direction::North => block.textures.front.clone(), 
                    Direction::East => block.textures.right.clone(), 
                    Direction::West => block.textures.left.clone(), 
                };
                // Add vertices for this face
                for i in 0..4 {
                    vertices.push(Vertex::new(
                        [
                            world_x as f32 + positions[i].0,
                            world_y as f32 + positions[i].1,
                            world_z as f32 + positions[i].2,
                        ],
                        normal,
                        [uvs[i].0, uvs[i].1],
                        atlas.get_texture_index(face_texture.as_str()).unwrap()
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
    
    fn should_render_face(
        &self,
        x: i32,
        y: i32,
        z: i32,
        adjacent_chunks: Option<&AdjacentChunks>,
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

        // If we're at a chunk boundary, check the adjacent chunk
        if let Some(adjacent_chunks) = adjacent_chunks {
            let (chunk, new_x, new_y, new_z) = match direction {
                Direction::North if z < 0 => {
                    if let Some(chunk) = adjacent_chunks.north {
                        (chunk, x as usize, y as usize, CHUNK_SIZE_Z - 1)
                    } else {
                        return true;
                    }
                }
                Direction::South if z >= CHUNK_SIZE_Z as i32 => {
                    if let Some(chunk) = adjacent_chunks.south {
                        (chunk, x as usize, y as usize, 0)
                    } else {
                        return true;
                    }
                }
                Direction::East if x >= CHUNK_SIZE_X as i32 => {
                    if let Some(chunk) = adjacent_chunks.east {
                        (chunk, 0, y as usize, z as usize)
                    } else {
                        return true;
                    }
                }
                Direction::West if x < 0 => {
                    if let Some(chunk) = adjacent_chunks.west {
                        (chunk, CHUNK_SIZE_X - 1, y as usize, z as usize)
                    } else {
                        return true;
                    }
                }
                Direction::Up if y >= CHUNK_SIZE_Y as i32 => {
                    if let Some(chunk) = adjacent_chunks.up {
                        (chunk, x as usize, 0, z as usize)
                    } else {
                        return true;
                    }
                }
                Direction::Down if y < 0 => {
                    if let Some(chunk) = adjacent_chunks.down {
                        (chunk, x as usize, CHUNK_SIZE_Y - 1, z as usize)
                    } else {
                        return true;
                    }
                }
                _ => return false,
            };

            chunk.get_block(new_x, new_y, new_z)== None
        } else {
            // If no adjacent chunks are provided, render the face
            true
        }
    }

    // Generate mesh data for the chunk
    fn generate_mesh_data(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        adjacent_chunks: Option<&AdjacentChunks>,
        block_registry: &Arc<BlockRegistry>,
        texture_atlas_bind_group_layout: &BindGroupLayout
    ) -> (Vec<Vertex>, Vec<u16>, ChunkTextureAtlas) {
        let mut vertices = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let atlas = ChunkTextureAtlas::new(device, queue, self, block_registry, texture_atlas_bind_group_layout);
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    if self.get_block(x, y, z) == None {
                        continue;
                    }
                    self.add_block_faces(x, y, z, &mut vertices, &mut indices, adjacent_chunks, block_registry, &atlas);
                }
            }
        }
        (vertices, indices, atlas)
    }
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

    fn update_mesh(&mut self, chunk_pos: IVec3, vertices: &[Vertex], indices: &[u16], atlas: &ChunkTextureAtlas) {
        let buffers = if let Some((existing_buffers, _, _)) = self.active_meshes.remove(&chunk_pos) {
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

pub struct ChunkManager {
    chunks: HashMap<IVec3, Chunk>,
    mesh_manager: ChunkMeshManager,
    update_queue: VecDeque<IVec3>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    block_registry: Arc<BlockRegistry>
}

impl ChunkManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, block_registry: Arc<BlockRegistry>) -> Self {
        Self {
            chunks: HashMap::new(),
            mesh_manager: ChunkMeshManager::new(device.clone(), queue.clone()),
            update_queue: VecDeque::new(),
            device,
            queue,
            block_registry
        }
    }

    pub fn update_chunk(
        &mut self,
        position: IVec3,
        blocks: Vec<Option<BlockType>>,
    ) {
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
        let texture_atlas_bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        for _ in 0..UPDATES_PER_FRAME {
            if let Some(chunk_pos) = self.update_queue.pop_front() {
                if let Some(chunk) = self.chunks.get(&chunk_pos) {
                    let adjacent_chunks = self.get_adjacent_chunks(chunk_pos);
                    let (vertices, indices, atlas) = chunk.generate_mesh_data(&self.device, &self.queue, Some(&adjacent_chunks), &self.block_registry, &texture_atlas_bind_group_layout);

                    if vertices.is_empty() {
                        self.mesh_manager.remove_mesh(&chunk_pos);
                    } else {
                        self.mesh_manager
                            .update_mesh(chunk_pos, &vertices, &indices, &atlas);
                    }
                }
            } else {
                break;
            }
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, camera:&mut Camera) {
        for (chunk_pos, (buffers, index_count, atlas)) in &self.mesh_manager.active_meshes {
            // Skip chunks outside view frustum
            // if !view_frustum.contains_chunk(*chunk_pos) {
            //     continue;
            // }
            let chunk_bbox = BoundingBox::from_chunk_position(*chunk_pos);
            if !camera.is_in_frustum(&chunk_bbox){
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