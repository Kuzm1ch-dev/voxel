use std::sync::Arc;

use glam::IVec3;

use crate::{render::camera::Camera, world::chunk::{BlockType, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z}};

use super::chunk::ChunkManager;

pub struct World {
    chunk_manager: ChunkManager,
}

impl World {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
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

    pub fn create_initial_chunks(&mut self) {
        fn generate_test_chunk(
            chunk_x: i32,
            chunk_z: i32,
        ) -> Box<[[[BlockType; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]> {
            let mut blocks =
                Box::new([[[BlockType::Air; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]);

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
                            BlockType::Grass
                        } else if y > height - 4 {
                            BlockType::Dirt
                        } else {
                            BlockType::Stone
                        };
                    }
                }
            }

            blocks
        }

        for x in -3..=3 {
            for z in -3..=3 {
                let chunk_pos = IVec3::new(x, 0, z);
                let blocks = generate_test_chunk(x, z);
                self.chunk_manager.update_chunk(chunk_pos, blocks);
            }
        }
    }
}
