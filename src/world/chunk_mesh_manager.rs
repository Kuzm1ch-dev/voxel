use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex}};

use glam::IVec3;
use wgpu::BindGroupLayout;

use crate::{model::vertex::Vertex, render::texture_manager::ChunkTextureAtlas};

use super::{block_registry::BlockRegistry, chunk::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z}};

const POOL_INITIAL_SIZE: usize = 64;
const MAX_VERTICES_PER_CHUNK: usize = CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z * 24; // 24 vertices per block worst case
const MAX_INDICES_PER_CHUNK: usize = CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z * 36; // 36 indices per block worst case

pub fn generate_mesh_data(
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