use winit::event::{DeviceEvent, KeyEvent, MouseButton, ElementState};
use winit::keyboard::{KeyCode, PhysicalKey};
use voxel_engine::{InputHandler, Player};
use crate::player::GamePlayer;

pub struct GameInputHandler {
    keys_pressed: std::collections::HashSet<KeyCode>,
}

impl GameInputHandler {
    pub fn new() -> Self {
        Self {
            keys_pressed: std::collections::HashSet::new(),
        }
    }

    pub fn update_player(&self, player: &mut GamePlayer, dt: f32) {
        if self.keys_pressed.contains(&KeyCode::KeyW) {
            player.move_forward(dt);
        }
        if self.keys_pressed.contains(&KeyCode::KeyS) {
            player.move_backward(dt);
        }
        if self.keys_pressed.contains(&KeyCode::KeyA) {
            player.move_left(dt);
        }
        if self.keys_pressed.contains(&KeyCode::KeyD) {
            player.move_right(dt);
        }
        if self.keys_pressed.contains(&KeyCode::Space) {
            player.move_up(dt);
        }
        if self.keys_pressed.contains(&KeyCode::ShiftLeft) {
            player.move_down(dt);
        }
    }
}

impl InputHandler for GameInputHandler {
    fn handle_keyboard(&mut self, event: KeyEvent) -> bool {
        if let PhysicalKey::Code(keycode) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    self.keys_pressed.insert(keycode);
                }
                ElementState::Released => {
                    self.keys_pressed.remove(&keycode);
                }
            }
            true
        } else {
            false
        }
    }

    fn handle_mouse_button(&mut self, _button: MouseButton, _state: ElementState) -> bool {
        true
    }

    fn handle_mouse_motion(&mut self, _delta_x: f32, _delta_y: f32) -> bool {
        true
    }

    fn handle_device_event(&mut self, _event: DeviceEvent) -> bool {
        true
    }
}