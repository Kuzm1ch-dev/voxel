use std::sync::Arc;
use voxel_engine::Engine;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::game_state::VoxelGameState;
use crate::systems::render_system::RenderSystem;
use crate::systems::input_system::InputSystem;
use crate::systems::ui_system::UISystem;

#[derive(Default)]
pub struct Game<'window> {
    window: Option<Arc<Window>>,
    engine: Option<Engine<'window>>,
    game_state: Option<VoxelGameState>,
    ui_system: Option<UISystem>,
    render_system: Option<RenderSystem>,
    input_system: Option<InputSystem>,
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
            let mut engine = Engine::new(window.clone());
            for (path) in game_state.world.world.registry.get_texture_paths().iter(){
                engine.renderer.add_texture(path);
            }
            engine.lock_cursor();
            
            // Load block textures for UI
            // for path in texture_paths {
            //     engine.load_ui_texture(path);
            // }
            
            self.engine = Some(engine);
            self.game_state = Some(game_state);
            self.ui_system = Some(UISystem::new());
            self.render_system = Some(RenderSystem::new());
            self.input_system = Some(InputSystem::new());
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
                if let (Some(engine), Some(game_state), Some(ui_system), Some(render_system)) = 
                    (self.engine.as_mut(), self.game_state.as_mut(), self.ui_system.as_mut(), self.render_system.as_ref()) 
                {
                    game_state.update(0.016, ui_system.is_open);
                    
                    // Update cursor state based on UI
                    if ui_system.is_open {
                        engine.unlock_cursor();
                    } else {
                        engine.lock_cursor();
                    }
                    
                    let _ = render_system.render(engine, game_state, ui_system);
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {
                if let (Some(game_state), Some(ui_system), Some(input_system), Some(engine)) = 
                    (self.game_state.as_mut(), self.ui_system.as_mut(), self.input_system.as_ref(), self.engine.as_mut()) 
                {
                    let winit_event = winit::event::Event::WindowEvent {
                        window_id: _window_id,
                        event,
                    };
                    if let Some(input_event) = voxel_engine::input::process_winit_event(&winit_event) {
                        let screen_size = glam::Vec2::new(800.0, 600.0); // TODO: получать реальный размер
                        input_system.handle_input(&input_event, game_state, ui_system, screen_size, engine);
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
        if let (Some(game_state), Some(ui_system), Some(input_system), Some(engine)) = 
            (self.game_state.as_mut(), self.ui_system.as_mut(), self.input_system.as_ref(), self.engine.as_mut()) 
        {
            let winit_event = winit::event::Event::DeviceEvent {
                device_id,
                event,
            };
            if let Some(input_event) = voxel_engine::input::process_winit_event(&winit_event) {
                let screen_size = glam::Vec2::new(800.0, 600.0); // TODO: получать реальный размер
                input_system.handle_input(&input_event, game_state, ui_system, screen_size, engine);
            }
        }
    }
}