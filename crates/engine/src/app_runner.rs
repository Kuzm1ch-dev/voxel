use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::{Engine, app::GameApp, input};
use glam::Vec2;

pub struct AppRunner<T: GameApp> {
    window: Option<Arc<Window>>,
    engine: Option<Engine<'static>>,
    game_app: T,
    initialized: bool,
}

impl<T: GameApp> AppRunner<T> {
    pub fn new(game_app: T) -> Self {
        Self {
            window: None,
            engine: None,
            game_app,
            initialized: false,
        }
    }
}

impl<T: GameApp> ApplicationHandler for AppRunner<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("Voxel Game");
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("Failed to create window"),
            );
            self.window = Some(window.clone());
            
            let mut engine = Engine::new(window.clone());
            self.game_app.ready(&mut engine);
            self.engine = Some(engine);
            self.initialized = true;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if !self.initialized {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(engine), Some(window)) = (self.engine.as_mut(), self.window.as_ref()) {
                    self.game_app.resize(engine, new_size);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(engine) = self.engine.as_mut() {
                    self.game_app.update(engine, 0.016);
                    self.game_app.render(engine);
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {
                if let Some(engine) = self.engine.as_mut() {
                    let winit_event = winit::event::Event::WindowEvent {
                        window_id: _window_id,
                        event,
                    };
                    if let Some(input_event) = input::process_winit_event(&winit_event) {
                        let screen_size = Vec2::new(800.0, 600.0); // TODO: get real size
                        self.game_app.input_event(engine, &input_event, screen_size);
                    }
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if !self.initialized {
            return;
        }

        if let Some(engine) = self.engine.as_mut() {
            let winit_event = winit::event::Event::DeviceEvent {
                device_id,
                event,
            };
            if let Some(input_event) = input::process_winit_event(&winit_event) {
                let screen_size = Vec2::new(800.0, 600.0); // TODO: get real size
                self.game_app.input_event(engine, &input_event, screen_size);
            }
        }
    }
}