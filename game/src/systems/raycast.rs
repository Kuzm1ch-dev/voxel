use glam::Vec3;
use crate::voxel_world::VoxelWorld;

#[derive(Debug, Clone, Copy)]
pub enum BlockFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub block_pos: (i32, i32, i32),
    pub chunk_pos: (i32, i32),
    pub face: BlockFace,
    pub hit_point: Vec3,
    pub distance: f32,
}

pub struct Raycast;

impl Raycast {
    pub fn cast_ray(origin: Vec3, direction: Vec3, max_distance: f32, world: &VoxelWorld) -> Option<RaycastHit> {
        let mut current_pos = origin;
        let step = direction.normalize() * 0.1;
        let mut distance = 0.0;
        
        while distance < max_distance {
            let block_pos = (
                current_pos.x.floor() as i32,
                current_pos.y.floor() as i32,
                current_pos.z.floor() as i32,
            );
            
            // Check if we hit a solid block (not air)
            if world.get_block_at(block_pos) != "air" {
                let face = Self::get_hit_face(current_pos - step, current_pos, block_pos);
                let chunk_pos = Self::world_to_chunk_pos(block_pos);
                
                return Some(RaycastHit {
                    block_pos,
                    chunk_pos,
                    face,
                    hit_point: current_pos,
                    distance,
                });
            }
            
            current_pos += step;
            distance += 0.1;
        }
        
        None
    }
    
    pub fn get_adjacent_block_pos(hit: &RaycastHit) -> (i32, i32, i32) {
        let (x, y, z) = hit.block_pos;
        match hit.face {
            BlockFace::Top => (x, y + 1, z),
            BlockFace::Bottom => (x, y - 1, z),
            BlockFace::North => (x, y, z - 1),
            BlockFace::South => (x, y, z + 1),
            BlockFace::East => (x + 1, y, z),
            BlockFace::West => (x - 1, y, z),
        }
    }

    
    fn get_hit_face(_prev_pos: Vec3, current_pos: Vec3, block_pos: (i32, i32, i32)) -> BlockFace {
        let block_center = Vec3::new(
            block_pos.0 as f32 + 0.5,
            block_pos.1 as f32 + 0.5,
            block_pos.2 as f32 + 0.5,
        );
        
        let diff = current_pos - block_center;
        let abs_diff = diff.abs();
        
        if abs_diff.y > abs_diff.x && abs_diff.y > abs_diff.z {
            if diff.y > 0.0 { BlockFace::Top } else { BlockFace::Bottom }
        } else if abs_diff.x > abs_diff.z {
            if diff.x > 0.0 { BlockFace::East } else { BlockFace::West }
        } else {
            if diff.z > 0.0 { BlockFace::South } else { BlockFace::North }
        }
    }
    
    fn world_to_chunk_pos(world_pos: (i32, i32, i32)) -> (i32, i32) {
        const CHUNK_SIZE: i32 = 16;
        (
            world_pos.0.div_euclid(CHUNK_SIZE),
            world_pos.2.div_euclid(CHUNK_SIZE),
        )
    }
}