use std::{array, path::Path, sync::{Arc, Mutex}};

use glam::IVec3;
use wgpu::{naga::Block, BindGroupLayout, Device, Queue};

use crate::{render::camera::Camera, world::{block::BlockType, chunk::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z}}};

use super::{block::BlockTextures, block_registry::{self, BlockRegistry}, chunk::{self, ChunkManager}};

pub struct World {
    block_registry: Arc<Mutex<BlockRegistry>>,
    chunk_manager: ChunkManager,
}

impl World {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let block_registry = BlockRegistry::new(&device);
        let _block_registry = Arc::new(Mutex::new(block_registry));
        Self {
            block_registry: _block_registry.clone(),
            chunk_manager: ChunkManager::new(device, queue, _block_registry),
        }
    }

    pub fn register_blocks(&mut self, device: &Device, queue: &Queue){
        let mut block_registry_lock = self.block_registry.lock().unwrap();
        let _ = block_registry_lock.register_block(
            &device,
            &queue,
            "grass",
            BlockTextures::uniform("grass".to_string()),
            Path::new("assets/textures/blocks"),
        );
        let _ = block_registry_lock.register_block(
            &device,
            &queue,
            "dirt",
            BlockTextures::uniform("dirt".to_string()),
            Path::new("assets/textures/blocks"),
        );
        let _ = block_registry_lock.register_block(
            &device,
            &queue,
            "stone",
            BlockTextures::new(
                "stone_u".to_string(),
                "stone_b".to_string(),
                "stone_n".to_string(),
                "stone_s".to_string(),
                "stone_w".to_string(),
                "stone_e".to_string(),
            ),
            Path::new("assets/textures/blocks"),
        );
    }

    pub fn get_chunk_manager(&self) -> &ChunkManager {
        &self.chunk_manager
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, camera: &mut Camera) {
        self.chunk_manager.render(render_pass, camera);
    }

    pub fn process_mesh_updates(&mut self) {
        self.chunk_manager.process_mesh_updates();
    }

    pub fn create_initial_chunks(&mut self, size: i32) {
        for x in -size..=size {
            for z in -size..=size {
                let chunk_pos = IVec3::new(x, 0, z);
                let blocks = self.generate_test_chunk(x, z);
                self.chunk_manager.update_chunk(chunk_pos, blocks);
            }
        }
    }

    pub fn remove_block(&mut self, block_pos: IVec3){
        let chunk_pos = Chunk::get_chunk_position(block_pos);
        let block_pos = Chunk::get_block_position(block_pos);
        let chunk = self.chunk_manager.get_chunk(chunk_pos);
        if let Some(chunk) = chunk {
            let mut chunk_lock = chunk.lock().unwrap();
            chunk_lock.set_block(block_pos.x as usize, block_pos.y as usize, block_pos.z as usize, None);
        }
    }

    pub fn ray_cast(&mut self, from: glam::Vec3, direction: glam::Vec3, distance: f32) -> Option<(IVec3, BlockType)> {
        let mut current_pos = from;
        let target = from + direction * distance;
        while current_pos.distance(target) > 0.05 {
            let block_pos_w = current_pos.floor().as_ivec3();
            let block_pos = Chunk::get_block_position(block_pos_w);
            let chunk_pos = Chunk::get_chunk_position(block_pos);
            let chunk = self.chunk_manager.get_chunk(chunk_pos);
            if let Some(chunk) = chunk {
                let chunk_lock = chunk.lock().unwrap();
                let block = chunk_lock.get_block(block_pos.x as usize, block_pos.y as usize, block_pos.z as usize);
                if let Some(block) = block {
                    return Some((block_pos, block.clone()));
                }
            }
            current_pos += direction * 0.01;
        }
        None
    }

    fn generate_test_chunk(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Vec<Option<BlockType>> {
        let mut blocks = vec![
            None;
            CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z
        ];
        // Generate some test terrain
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                // Create a simple heightmap using sine waves
                let world_x = (chunk_x * CHUNK_SIZE_X as i32 + x as i32) as f32;
                let world_z = (chunk_z * CHUNK_SIZE_Z as i32 + z as i32) as f32;

                let height =
                    ((world_x * 0.1).sin() * 5.0 + (world_z * 0.1).cos() * 5.0 + 32.0) as usize;

                // Fill blocks up to the height
                for y in 0..height.min(CHUNK_SIZE_Y) {
                    let index = Chunk::get_index(x, y, z);
                    let block_registry_lock = self.block_registry.lock().unwrap();
                    blocks[index] = if y == height - 1 {
                        block_registry_lock.get_block("grass")
                    } else if y > height - 4 {
                        block_registry_lock.get_block("dirt")
                    } else {
                        block_registry_lock.get_block("stone")
                    };
                    drop(block_registry_lock)
                }
            }
        }

        blocks
    }
}
