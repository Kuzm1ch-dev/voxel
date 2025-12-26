use std::{collections::{HashMap, HashSet, VecDeque}, sync::{mpsc::{self, channel, Receiver, Sender}, Arc, Mutex, RwLock}, thread, time::Instant};

use glam::IVec3;
use noise::{NoiseFn, Perlin};

use voxel_engine::Vertex;

use super::{block::{BlockTextures, BlockType}, block_registry::BlockRegistry, chunk::{AdjacentChunks, Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z}, chunk_mesh_manager::{generate_mesh_data, ChunkMeshManager}};

pub struct ChunkManager {
    pub chunks: HashMap<IVec3, Chunk>,
    mesh_manager: ChunkMeshManager,
    update_queue: VecDeque<IVec3>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    block_registry: Arc<RwLock<BlockRegistry>>,
    mesh_generation_threads: Vec<thread::JoinHandle<()>>,
    mesh_sender: Sender<MeshGenerationTask>,
    mesh_receiver: Receiver<MeshGenerationResult>,
}

struct MeshGenerationTask {
    chunk_pos: IVec3,
    chunk: Arc<Mutex<Chunk>>,
    neighbors: [Option<Arc<Mutex<Chunk>>>; 4],
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

fn start_mesh_generation_threads(
    task_receiver: Arc<Mutex<mpsc::Receiver<MeshGenerationTask>>>,
    result_sender: mpsc::Sender<MeshGenerationResult>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    block_registry: Arc<RwLock<BlockRegistry>>,
    thread_count: usize,
) -> Vec<thread::JoinHandle<()>> {
    let mut handles = Vec::with_capacity(thread_count);
    
    for thread_id in 0..thread_count {
        let task_receiver = task_receiver.clone();
        let result_sender = result_sender.clone();
        let device_clone = device.clone();
        let queue_clone = queue.clone();
        let block_registry_clone = block_registry.clone();
        
        let handle = thread::spawn(move || {
            println!("Mesh generation thread {} started", thread_id);
            
            loop {
                // Get a task from the shared receiver
                let task = {
                    let receiver = task_receiver.lock().unwrap();
                    match receiver.recv() {
                        Ok(task) => task,
                        Err(_) => break, // Channel closed, exit thread
                    }
                };
                let chunk_lock = task.chunk.lock().unwrap();
                let adjacent_chunks = task.neighbors.clone();
                let block_registry_clone_lock = block_registry_clone.read().unwrap();
                let texture_atlas_bind_group_layout =
                    device_clone.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("Chunk Texture Bind Group Layout"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
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
                let now = Instant::now();
                let (vertices, indices, atlas) = generate_mesh_data(
                    &chunk_lock,
                    &device_clone,
                    &queue_clone,
                    adjacent_chunks,
                    &block_registry_clone_lock,
                    &texture_atlas_bind_group_layout,
                );
                println!("Mesh generation took {}ms", now.elapsed().as_millis());
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
                    break; // Receiver dropped, exit thread
                }
            }
            println!("Mesh generation thread {} exiting", thread_id);
        });
        
        handles.push(handle);
    }
    
    handles
}


impl ChunkManager {
    const RENDER_DISTANCE: i32 = 5;
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        block_registry: Arc<RwLock<BlockRegistry>>,
    ) -> Self {
        let (task_sender, task_receiver) = channel::<MeshGenerationTask>(); // создаем канал
        let (result_sender, result_receiver) = channel::<MeshGenerationResult>();
        let thread_count = 2; 
        let task_receiver = Arc::new(Mutex::new(task_receiver));
        let mesh_generation_threads = start_mesh_generation_threads(
            task_receiver.clone(),
            result_sender,
            device.clone(),
            queue.clone(),
            block_registry.clone(),
            thread_count,
        );
        Self {
            chunks: HashMap::new(),
            mesh_manager: ChunkMeshManager::new(device.clone(), queue.clone()),
            update_queue: VecDeque::new(),
            device,
            queue,
            block_registry,
            mesh_generation_threads,
            mesh_sender: task_sender,
            mesh_receiver: result_receiver,
        }
    }

    pub fn update_visible_chunks(&mut self, camera_position: IVec3) {
        // Convert camera world position to chunk coordinates
        let camera_chunk_pos = Chunk::get_chunk_position(camera_position);
        
        // Keep track of chunks we want to keep
        let mut chunks_to_keep = HashSet::new();
        
        // Generate chunks in a square around the camera
        for x in -Self::RENDER_DISTANCE..=Self::RENDER_DISTANCE {
            for z in -Self::RENDER_DISTANCE..=Self::RENDER_DISTANCE {
                let chunk_pos = camera_chunk_pos + IVec3::new(x, 0, z);
                chunks_to_keep.insert(chunk_pos);
                
                // If chunk doesn't exist, create it
                if !self.chunks.contains_key(&chunk_pos) && !self.update_queue.contains(&chunk_pos){
                    let mut new_chunk = Chunk::new(chunk_pos);
                    // Here you would add your terrain generation logic
                    self.generate_terrain(&mut new_chunk);
                    self.chunks.insert(chunk_pos, new_chunk);
                    self.update_chunk_by_pos(chunk_pos);
                }
            }
        }
        
        // Remove chunks that are too far away
        let chunks_to_unload: Vec<IVec3> = self.chunks.keys()
            .filter(|pos| !chunks_to_keep.contains(pos))
            .cloned()
            .collect();
            
        for pos in chunks_to_unload {
            self.unload_chunk(&pos);
        }
    }
    
    fn generate_terrain(&self, chunk: &mut Chunk) {
        let perlin = Perlin::new(12133);
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                // Convert chunk coordinates to world coordinates
                let world_x = chunk.get_position().x * CHUNK_SIZE_X as i32 + x as i32;
                let world_z = chunk.get_position().z * CHUNK_SIZE_Z as i32 + z as i32;

                // Generate height using Perlin noise
                let height = (perlin.get([world_x as f64 * 0.01, world_z as f64 * 0.01]) * 32.0
                    + 64.0) as usize;

                for y in 0..CHUNK_SIZE_Y {
                    if y < height - 4 {
                        chunk.set_block(x, y, z, Some(
                            BlockType::new("stone".to_string(), BlockTextures::uniform("stone".to_string()))
                        ));
                    } else if y < height {
                        chunk.set_block(x, y, z, Some(
                            BlockType::new("dirt".to_string(), BlockTextures::uniform("dirt".to_string()))
                        ));
                    } else if y == height {
                        chunk.set_block(x, y, z, Some(
                            BlockType::new("grass".to_string(), BlockTextures::uniform("grass".to_string()))
                        ));
                    }
                }
            }
        }
    }


    pub fn add_chunk(&mut self, position: IVec3, chunk: Chunk) {
        self.chunks.insert(position, chunk);
        self.update_chunk_by_pos(position);
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
            if !self.chunks.contains_key(&adj_pos) {
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

    pub fn process_mesh_updates(&mut self, profiler: &mut Profiler) {
        profiler.begin_scope("Mesh Receiver");
        while let Ok(result) = self.mesh_receiver.try_recv() {
            if result.mesh_data.vertices.is_empty() {
                self.mesh_manager.remove_mesh(&result.chunk_pos);
            } else {
                self.mesh_manager.update_mesh(
                    result.chunk_pos,
                    &result.mesh_data.vertices,
                    &result.mesh_data.indices,
                    &result.mesh_data.atlas,
                );
            }
        }
        profiler.end_scope("Mesh Receiver");
        profiler.begin_scope("Mesh Sender");
        if let Some(chunk_pos) = self.update_queue.pop_front() {
            if let Some(chunk) = self.chunks.get(&chunk_pos) {
                let adjacent_chunks: AdjacentChunks<'_> = self.get_adjacent_chunks(chunk_pos);
                let _ = self.mesh_sender.send(MeshGenerationTask {
                    chunk_pos,
                    chunk: Arc::new(Mutex::new(chunk.clone())),
                    neighbors: [
                        adjacent_chunks
                            .north
                            .map(|c| Arc::new(Mutex::new(c.clone()))),
                        adjacent_chunks
                            .south
                            .map(|c| Arc::new(Mutex::new(c.clone()))),
                        adjacent_chunks
                            .east
                            .map(|c| Arc::new(Mutex::new(c.clone()))),
                        adjacent_chunks
                            .west
                            .map(|c| Arc::new(Mutex::new(c.clone())))
                    ],
                });
            }
        }
        profiler.end_scope("Mesh Sender");
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, camera: &mut Camera) {
        for (chunk_pos, (buffers, index_count, atlas)) in &self.mesh_manager.active_meshes {
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

    pub fn unload_chunk(&mut self, position: &IVec3) {
        self.chunks.remove(position);
        self.mesh_manager.remove_mesh(position);
    }

    fn get_adjacent_chunks(&self, position: IVec3) -> AdjacentChunks {
        AdjacentChunks::new(            
            self.chunks.get(&(position + IVec3::new(0, 0, -1))),
             self.chunks.get(&(position + IVec3::new(0, 0, 1))),
             self.chunks.get(&(position + IVec3::new(1, 0, 0))),
             self.chunks.get(&(position + IVec3::new(-1, 0, 0)))
        )
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