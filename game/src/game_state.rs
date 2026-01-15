use glam::{Vec2, Vec3};
use voxel_engine::{Engine, InputEvent};
use crate::player::GamePlayer;
use crate::systems::raycast::Raycast;
use crate::world::world::World;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

pub struct GameState {
    pub player: GamePlayer,
    pub world: World,
    pressed_keys: HashSet<KeyCode>,
    mouse_position: Option<glam::Vec2>,
}

impl GameState {
    pub fn new(engine: &mut Engine) -> Self {
        Self {
            player: GamePlayer::new(Vec3::new(0.0, 38.0, 0.0)),
            world: World::new(engine),
            pressed_keys: HashSet::new(),
            mouse_position: None,
        }
    }

    pub fn get_mouse_position(&self) -> Option<glam::Vec2> {
        self.mouse_position
    }
    
    pub fn set_mouse_position(&mut self, pos: glam::Vec2) {
        self.mouse_position = Some(pos);
    }

    pub fn update(&mut self, engine: &Engine, dt: f32, ui_open: bool) {
        // Handle continuous key presses only if UI is not open
        if !ui_open {
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

    pub fn handle_input(&mut self, engine: &Engine, input: &InputEvent, ui_open: bool) {
        match input {
            InputEvent::KeyPressed(key) => {
                self.pressed_keys.insert(*key);
            }
            InputEvent::KeyReleased(key) => {
                self.pressed_keys.remove(key);
            }
            InputEvent::MouseMotion(dx, dy) => {
                if !ui_open {
                    self.player.look(*dx, -*dy);
                }
            }
            InputEvent::CursorMoved(x, y) => {
                // Нормализуем координаты к диапазону 0-1
                self.set_mouse_position(glam::Vec2::new(*x, *y));
            }
            InputEvent::MouseButton(button, state) => {
                if !ui_open && *state == winit::event::ElementState::Pressed {
                    match button {
                        winit::event::MouseButton::Left => {
                            // Break block using raycast
                            let ray_pos = self.player.get_camera_position();
                            let ray_dir = (self.player.get_camera_target() - ray_pos).normalize();
                            
                            if let Some(hit) = Raycast::cast_ray(ray_pos, ray_dir, 10.0, &self.world) {
                                self.world.break_block(engine, hit.block_pos);
                            }
                        }
                        winit::event::MouseButton::Right => {
                            // Place block using raycast
                            let ray_pos = self.player.get_camera_position();
                            let ray_dir = (self.player.get_camera_target() - ray_pos).normalize();
                            
                            if let Some(hit) = Raycast::cast_ray(ray_pos, ray_dir, 10.0, &self.world) {
                                let place_pos = Raycast::get_adjacent_block_pos(&hit);
                                self.world.place_block(engine, place_pos, "example:ruby_block");
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}