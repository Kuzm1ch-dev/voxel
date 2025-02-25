use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex}};

use glam::IVec3;
use wgpu::BindGroupLayout;

use crate::{model::vertex::Vertex, render::texture_manager::ChunkTextureAtlas};

use super::{block_registry::BlockRegistry, chunk::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z}};

const POOL_INITIAL_SIZE: usize = 64;
const MAX_VERTICES_PER_CHUNK: usize = CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z * 24; // 24 vertices per block worst case
const MAX_INDICES_PER_CHUNK: usize = CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z * 36; // 36 indices per block worst case

fn create_block_cache(chunk: &Chunk, adjacent_chunks: &[Option<Arc<Mutex<Chunk>>>; 4]) -> Vec<bool> {
    // Create a cache that's larger than the chunk to include borders from adjacent chunks
    // This avoids boundary checks during mesh generation
    let cache_size_x = CHUNK_SIZE_X + 2;
    let cache_size_y = CHUNK_SIZE_Y + 2;
    let cache_size_z = CHUNK_SIZE_Z + 2;
    let total_size = cache_size_x * cache_size_y * cache_size_z;
    
    let mut cache: Vec<bool> = vec![false; total_size];
    
    // Fill the cache with block IDs from the main chunk
    for x in 0..CHUNK_SIZE_X {
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                let block = chunk.get_block(x, y, z);
                let cache_index = get_cache_index(x + 1, y + 1, z + 1, cache_size_x, cache_size_y);
                cache[cache_index] = block.is_some();
            }
        }
    }
    
    // Fill the borders with blocks from adjacent chunks
    // For simplicity, let's assume adjacent_chunks are in order: -X, +X, -Z, +Z
    
    // -X face
    if let Some(adj_chunk) = &adjacent_chunks[0] {
        if let Ok(adj_chunk) = adj_chunk.lock() {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let block = adj_chunk.get_block(CHUNK_SIZE_X - 1, y, z);
                    let cache_index = get_cache_index(0, y + 1, z + 1, cache_size_x, cache_size_y);
                    cache[cache_index] = block.is_some();
                }
            }
        }
    }
    
    // +X face
    if let Some(adj_chunk) = &adjacent_chunks[1] {
        if let Ok(adj_chunk) = adj_chunk.lock() {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let block = adj_chunk.get_block(0, y, z);
                    let cache_index = get_cache_index(CHUNK_SIZE_X + 1, y + 1, z + 1, cache_size_x, cache_size_y);
                    cache[cache_index] = block.is_some();
                }
            }
        }
    }
    
    // -Z face
    if let Some(adj_chunk) = &adjacent_chunks[2] {
        if let Ok(adj_chunk) = adj_chunk.lock() {
            for x in 0..CHUNK_SIZE_X {
                for y in 0..CHUNK_SIZE_Y {
                    let block = adj_chunk.get_block(x, y, CHUNK_SIZE_Z - 1);
                    let cache_index = get_cache_index(x + 1, y + 1, 0, cache_size_x, cache_size_y);
                    cache[cache_index] = block.is_some();
                }
            }
        }
    }
    
    // +Z face
    if let Some(adj_chunk) = &adjacent_chunks[3] {
        if let Ok(adj_chunk) = adj_chunk.lock() {
            for x in 0..CHUNK_SIZE_X {
                for y in 0..CHUNK_SIZE_Y {
                    let block = adj_chunk.get_block(x, y, 0);
                    let cache_index = get_cache_index(x + 1, y + 1, CHUNK_SIZE_Z + 1, cache_size_x, cache_size_y);
                    cache[cache_index] = block.is_some();
                }
            }
        }
    }
    
    cache
}

#[inline]
fn get_cache_index(x: usize, y: usize, z: usize, cache_size_x: usize, cache_size_y: usize) -> usize {
    x + y * cache_size_x + z * cache_size_x * cache_size_y
}

#[inline]
fn get_block_from_cache(x: usize, y: usize, z: usize, cache: &[bool], cache_size_x: usize, cache_size_y: usize) -> bool {
    // Add 1 to coordinates because our cache has a 1-block border
    let cache_x = x + 1;
    let cache_y = y + 1;
    let cache_z = z + 1;
    let index = get_cache_index(cache_x, cache_y, cache_z, cache_size_x, cache_size_y);
    cache[index]
}


pub fn generate_mesh_data(
    chunk: &Chunk,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    adjacent_chunks: [Option<Arc<Mutex<Chunk>>>; 4],
    block_registry: &BlockRegistry,
    texture_atlas_bind_group_layout: &BindGroupLayout,
) -> (Vec<Vertex>, Vec<u16>, ChunkTextureAtlas) {
    let mut vertices = Vec::with_capacity(MAX_VERTICES_PER_CHUNK / 4);
    let mut indices: Vec<u16> = Vec::with_capacity(MAX_INDICES_PER_CHUNK / 4);
    
    let atlas = ChunkTextureAtlas::new(
        device,
        queue,
        &chunk,
        block_registry,
        texture_atlas_bind_group_layout,
    );
    
    // Create the block cache
    let block_cache = create_block_cache(chunk, &adjacent_chunks);
    let cache_size_x = CHUNK_SIZE_X + 2;
    let cache_size_y = CHUNK_SIZE_Y + 2;
    
    // Use the cache for mesh generation
    for x in 0..CHUNK_SIZE_X {
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                if get_block_from_cache(x, y, z, &block_cache, cache_size_x, cache_size_y) {
                    // We need to modify add_block_faces to work with block_id instead of getting the block again
                    // For now, we'll continue using the original method
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
    }
    
    (vertices, indices, atlas)
}

pub struct BufferPair {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

struct MeshPool {
    available_buffers: VecDeque<BufferPair>,
    device: Arc<wgpu::Device>,
}

impl MeshPool {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
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

    pub fn create_buffer_pair(device: &wgpu::Device) -> BufferPair {
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

pub struct ChunkMeshManager {
    mesh_pool: MeshPool,
    pub active_meshes: HashMap<IVec3, (BufferPair, u32, ChunkTextureAtlas)>, // (buffers, index_count)
    queue: Arc<wgpu::Queue>,
}

impl ChunkMeshManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            mesh_pool: MeshPool::new(device),
            active_meshes: HashMap::new(),
            queue,
        }
    }

    pub fn update_mesh(
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

    pub fn remove_mesh(&mut self, chunk_pos: &IVec3) {
        if let Some((buffers, _, _)) = self.active_meshes.remove(chunk_pos) {
            self.mesh_pool.return_buffers(buffers);
        }
    }
}