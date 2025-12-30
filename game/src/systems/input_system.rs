use voxel_engine::{InputEvent, Engine};
use winit::keyboard::KeyCode;
use crate::{game_state::VoxelGameState, systems::ui_system::UISystem};

pub struct InputSystem;

impl InputSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_input(&self, input: &InputEvent, game_state: &mut VoxelGameState, ui_system: &mut UISystem, screen_size: glam::Vec2, engine: &mut Engine) {
        match input {
            InputEvent::KeyPressed(key) => {
                if *key == KeyCode::KeyI {
                    ui_system.toggle();
                    return;
                }
            }
            InputEvent::MouseButton(button, state) => {
                if *state == winit::event::ElementState::Pressed {
                    if let Some(mouse_pos) = game_state.get_mouse_position() {
                        ui_system.handle_click(mouse_pos, screen_size, engine);
                    }
                    return;
                }
            }
            _ => {}
        }
        
        // Всегда передаем события игровому состоянию
        game_state.handle_input(input, ui_system.is_open);
    }
}