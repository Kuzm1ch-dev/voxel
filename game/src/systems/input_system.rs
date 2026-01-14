use voxel_engine::{InputEvent, Engine};
use winit::keyboard::KeyCode;
use crate::{game_state::GameState, systems::ui_system::UISystem};

pub struct InputSystem;

impl InputSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_input(&self, engine: &mut Engine, input: &InputEvent, game_state: &mut GameState, ui_system: &mut UISystem) {
        match input {
            InputEvent::KeyPressed(key) => {
                if *key == KeyCode::KeyI {
                    ui_system.toggle();
                }
            }
            InputEvent::MouseButton(button, state) => {
                if *state == winit::event::ElementState::Pressed {
                    if let Some(mouse_pos) = game_state.get_mouse_position() {
                        ui_system.handle_click(engine, mouse_pos);
                    }
                }
            }
            _ => {}
        }
        
        // Всегда передаем события игровому состоянию
        game_state.handle_input(engine, input, ui_system.is_open);
    }
}