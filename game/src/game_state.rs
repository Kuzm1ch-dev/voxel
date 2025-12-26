use glam::Vec3;
use voxel_engine::InputEvent;
use crate::player::GamePlayer;
use crate::voxel_world::VoxelWorld;
use crate::blocks::BlockType;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

pub struct VoxelGameState {
    pub player: GamePlayer,
    pub world: VoxelWorld,
    pressed_keys: HashSet<KeyCode>,
    pub ui_open: bool,
}

impl VoxelGameState {
    pub fn new() -> Self {
        Self {
            player: GamePlayer::new(Vec3::new(0.0, 0.0, 5.0)),
            world: VoxelWorld::new(),
            pressed_keys: HashSet::new(),
            ui_open: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Handle continuous key presses only if UI is not open
        if !self.ui_open {
            if self.pressed_keys.contains(&KeyCode::KeyW) {
                self.player.move_forward(dt);
            }
            if self.pressed_keys.contains(&KeyCode::KeyS) {
                self.player.move_backward(dt);
            }
            if self.pressed_keys.contains(&KeyCode::KeyA) {
                self.player.move_left(dt);
            }
            if self.pressed_keys.contains(&KeyCode::KeyD) {
                self.player.move_right(dt);
            }
            if self.pressed_keys.contains(&KeyCode::Space) {
                self.player.move_up(dt);
            }
            if self.pressed_keys.contains(&KeyCode::ShiftLeft) {
                self.player.move_down(dt);
            }
        }
        
        self.player.update(dt);
    }

    pub fn handle_input(&mut self, input: &InputEvent) {
        match input {
            InputEvent::KeyPressed(key) => {
                if *key == KeyCode::KeyI {
                    self.ui_open = !self.ui_open;
                    return;
                }
                self.pressed_keys.insert(*key);
            }
            InputEvent::KeyReleased(key) => {
                self.pressed_keys.remove(key);
            }
            InputEvent::MouseMotion(dx, dy) => {
                if !self.ui_open {
                    self.player.look(*dx, -*dy);
                }
            }
            InputEvent::MouseButton(button, state) => {
                if !self.ui_open && *state == winit::event::ElementState::Pressed {
                    match button {
                        winit::event::MouseButton::Left => {
                            // Break block
                            let ray_pos = self.player.get_camera_position();
                            let ray_dir = (self.player.get_camera_target() - ray_pos).normalize();
                            
                            // Simple raycast - check blocks in front of player
                            for i in 1..10 {
                                let check_pos = ray_pos + ray_dir * i as f32;
                                let block_pos = (
                                    check_pos.x.floor() as i32,
                                    check_pos.y.floor() as i32,
                                    check_pos.z.floor() as i32,
                                );
                                
                                if self.world.break_block(block_pos) {
                                    break;
                                }
                            }
                        }
                        winit::event::MouseButton::Right => {
                            // Place block
                            let ray_pos = self.player.get_camera_position();
                            let ray_dir = (self.player.get_camera_target() - ray_pos).normalize();
                            
                            for i in 1..10 {
                                let check_pos = ray_pos + ray_dir * i as f32;
                                let block_pos = (
                                    check_pos.x.floor() as i32,
                                    check_pos.y.floor() as i32,
                                    check_pos.z.floor() as i32,
                                );
                                
                                if self.world.place_block(block_pos, BlockType::STONE) {
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}