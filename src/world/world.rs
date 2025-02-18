use std::{array, sync::Arc};

use glam::IVec3;

use crate::{render::camera::Camera, world::{block::BlockType, chunk::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z}}};

use super::{block_registry::{self, BlockRegistry}, chunk::ChunkManager};

pub struct World {
    block_registry: Arc<BlockRegistry>,
    chunk_manager: ChunkManager,
}

impl World {
    pub fn new(block_registry: Arc<BlockRegistry>,device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            block_registry,
            chunk_manager: ChunkManager::new(device, queue),
        }
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
        for x in 0..=size {
            for z in 0..=size {
                println!("{} {}", x,z);
                let chunk_pos = IVec3::new(x, 0, z);
                println!("chunk_pos");
                let blocks = self.generate_test_chunk(x, z);
                println!("blocks after");
                self.chunk_manager.update_chunk(chunk_pos, blocks);
            }
        }
    }

    fn generate_test_chunk(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Box<[[[Option<BlockType>; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]> {
        println!("blocks start");
        let mut blocks = Box::new(array::from_fn(|_| {
            array::from_fn(|_| {
                array::from_fn(|_| {
                    None
                })
            })
        }));
        println!("blocks");
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
                    blocks[x][y][z] = if y == height - 1 {
                        self.block_registry.get_block("grass")
                    } else if y > height - 4 {
                        self.block_registry.get_block("dirt")
                    } else {
                        self.block_registry.get_block("stone")
                    };
                }
            }
        }

        blocks
    }
}
