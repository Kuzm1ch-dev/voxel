use std::sync::Arc;
use voxel_engine::Engine;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::game_state::VoxelGameState;
use crate::texture_loader::TextureLoader;
use glam::{Vec2, Vec4};

#[derive(Default)]
pub struct Game<'window> {
    window: Option<Arc<Window>>,
    engine: Option<Engine<'window>>,
    game_state: Option<VoxelGameState>,
}

impl<'window> ApplicationHandler for Game<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("Voxel Game");
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            let game_state = VoxelGameState::new();
            let texture_paths = game_state.world.world.registry.get_texture_paths();
            let mut engine = Engine::new_with_textures(window.clone(), texture_paths);
            engine.lock_cursor(); // Блокируем курсор сразу
            TextureLoader::load_block_textures(&mut engine);
            
            // Загружаем UI текстуры
            engine.load_ui_texture("game/assets/textures/block/dirt.png");
            
            self.engine = Some(engine);
            self.game_state = Some(game_state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(engine), Some(window)) =
                    (self.engine.as_mut(), self.window.as_ref())
                {
                    engine.resize(new_size);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(engine), Some(game_state)) = 
                    (self.engine.as_mut(), self.game_state.as_mut()) 
                {
                    game_state.update(0.016);
                    
                    // Update cursor state based on UI
                    if game_state.ui_open {
                        engine.unlock_cursor();
                    } else {
                        engine.lock_cursor();
                    }
                    
                    let player = &game_state.player;
                    engine.update_camera(
                        player.get_camera_position(),
                        player.get_camera_target(),
                        player.get_camera_up()
                    );
                    
                    // Clear and setup UI
                    engine.clear_ui();
                    
                    // Always show coordinates in top-left
                    let pos = player.get_camera_position();
                    engine.add_ui_text(&format!("X: {:.1}", pos.x), Vec2::new(0.02, 0.02), 1.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
                    engine.add_ui_text(&format!("Y: {:.1}", pos.y), Vec2::new(0.02, 0.06), 1.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
                    engine.add_ui_text(&format!("Z: {:.1}", pos.z), Vec2::new(0.02, 0.10), 1.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
                    
                    if game_state.ui_open {
                        // Draw UI background
                        engine.add_ui_rect(Vec2::new(0.3, 0.3), Vec2::new(0.4, 0.4), Vec4::new(0.2, 0.2, 0.2, 0.9));
                        
                        // Draw bitmap text
                        engine.add_ui_text("INVENTORY", Vec2::new(0.32, 0.32), 2.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
                        engine.add_ui_text("PRESS I TO CLOSE", Vec2::new(0.32, 0.62), 1.0, Vec4::new(1.0, 1.0, 0.0, 1.0));
                        
                        // Show dirt texture
                        if let Some(dirt_id) = engine.get_ui_texture_id("game/assets/textures/block/dirt.png") {
                            engine.add_ui_image(Vec2::new(0.75, 0.1), Vec2::new(0.2, 0.2), dirt_id);
                        }
                    }
                    
                    game_state.world.render(engine);
                    let _ = engine.render();
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {
                if let Some(game_state) = self.game_state.as_mut() {
                    let winit_event = winit::event::Event::WindowEvent {
                        window_id: _window_id,
                        event,
                    };
                    if let Some(input_event) = voxel_engine::input::process_winit_event(&winit_event) {
                        game_state.handle_input(&input_event);
                    }
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(game_state) = self.game_state.as_mut() {
            let winit_event = winit::event::Event::DeviceEvent {
                device_id,
                event,
            };
            if let Some(input_event) = voxel_engine::input::process_winit_event(&winit_event) {
                game_state.handle_input(&input_event);
            }
        }
    }
}