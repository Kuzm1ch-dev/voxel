use std::{
    array, collections::HashSet, path::Path, sync::{Arc, Mutex, RwLock}
};

use glam::IVec3;
use noise::{NoiseFn, Perlin};
use wgpu::{naga::Block, BindGroupLayout, Device, Queue};

use crate::{
    render::{camera::Camera, profiler::{ProfileScope, Profiler}},
    world::{
        block::BlockType,
        chunk::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z},
    },
};

use super::{
    block::BlockTextures,
    block_registry::{self, BlockRegistry},
    chunk::{self}, chunk_manager::ChunkManager,
};

pub struct World {
    pub block_registry: Arc<RwLock<BlockRegistry>>,
    chunk_manager: ChunkManager,
}

impl World {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let block_registry = BlockRegistry::new(&device);
        let _block_registry = Arc::new(RwLock::new(block_registry));
        Self {
            block_registry: _block_registry.clone(),
            chunk_manager: ChunkManager::new(device, queue, _block_registry),
        }
    }

    pub fn register_blocks(&mut self, device: &Device, queue: &Queue) {
        let mut block_registry_lock = self.block_registry.write().unwrap();
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

    pub fn process_mesh_updates(&mut self, profiler: &mut Profiler) {
        self.chunk_manager.process_mesh_updates(profiler);
    }

    pub fn update_visible_chunks(&mut self, camera_position: IVec3){
        self.chunk_manager.update_visible_chunks(camera_position);
    }

    pub fn remove_block(&mut self, block_pos: IVec3) {
        let chunk_pos = Chunk::get_chunk_position(block_pos);
        let block_pos = Chunk::get_block_position(block_pos);
        let chunk = self.chunk_manager.get_chunk(chunk_pos);
        if let Some(chunk) = chunk {
            let mut chunk_lock = chunk.lock().unwrap();
            chunk_lock.set_block(
                block_pos.x as usize,
                block_pos.y as usize,
                block_pos.z as usize,
                None,
            );
        }
        self.chunk_manager.update_chunk_by_pos(chunk_pos);
    }

    pub fn break_block_in_chunk(&mut self, block_pos: IVec3, chunk_pos: IVec3) {
        let chunk = self.chunk_manager.get_chunk(chunk_pos);
        if let Some(chunk) = chunk {
            let mut chunk_lock = chunk.lock().unwrap();
            chunk_lock.set_block(
                block_pos.x as usize,
                block_pos.y as usize,
                block_pos.z as usize,
                None,
            );
        }
        self.chunk_manager.update_chunk_by_pos(chunk_pos);
    }

    pub fn place_block_in_chunk(&mut self, block_pos: IVec3, chunk_pos: IVec3, block_type: BlockType) {
        let chunk = self.chunk_manager.get_chunk(chunk_pos);
        if let Some(chunk) = chunk {
            let mut chunk_lock = chunk.lock().unwrap();
            chunk_lock.set_block(
                block_pos.x as usize,
                block_pos.y as usize,
                block_pos.z as usize,
                Some(block_type),
            );
        }
        self.chunk_manager.update_chunk_by_pos(chunk_pos);
    }

    pub fn ray_cast(
        &mut self,
        from: glam::Vec3,
        direction: glam::Vec3,
        distance: f32,
    ) -> Option<(IVec3,IVec3, IVec3, IVec3, BlockType)> {
        let mut last_pos = from;
        let mut current_pos = from;
        let target = from + direction * distance;
        while current_pos.distance(target) > 0.05 {
            let block_pos_w = current_pos.floor().as_ivec3();
            let block_pos = Chunk::get_block_position(block_pos_w);
            let chunk_pos = Chunk::get_chunk_position(block_pos_w);
            let chunk = self.chunk_manager.get_chunk(chunk_pos);
            if let Some(chunk) = chunk {
                let chunk_lock = chunk.lock().unwrap();
                let block = chunk_lock.get_block(
                    block_pos.x as usize,
                    block_pos.y as usize,
                    block_pos.z as usize,
                );

                let diff = current_pos - last_pos;
                let normal = if diff.x.abs() > diff.y.abs() && diff.x.abs() > diff.z.abs() {
                    IVec3::new(diff.x.signum() as i32, 0, 0)
                } else if diff.y.abs() > diff.x.abs() && diff.y.abs() > diff.z.abs() {
                    IVec3::new(0, diff.y.signum() as i32, 0)  
                } else {
                    IVec3::new(0, 0, diff.z.signum() as i32)
                }; 

                if let Some(block) = block {
                    return Some((block_pos_w, block_pos, chunk_pos, normal,block.clone()));
                }
            }
            last_pos = current_pos;
            current_pos += direction * 0.01;
        }
        None
    }
}
