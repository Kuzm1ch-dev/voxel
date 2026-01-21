use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use glam::Vec2;

use crate::engine::input::process_winit_event;
use crate::engine::{Engine, GameApp};

pub struct AppRunner<T: GameApp> {
    window: Arc<Window>,
    engine: Box<Engine<'static>>,
    game_app: T,
    initialized: bool,
}

impl<T: GameApp> AppRunner<T> {
    pub fn new(game_app: T, window: Arc<Window>) -> Self {
        let engine = Engine::new(window.clone());
        Self {
            window: window,
            engine: Box::new(engine),
            game_app,
            initialized: false,
        }
    }
}

impl<T: GameApp> ApplicationHandler for AppRunner<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.game_app.ready(self.engine.as_mut());
        self.initialized = true;
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
                self.game_app.resize(self.engine.as_mut(), new_size);
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.game_app.update(self.engine.as_mut(), 0.016);
                self.game_app.render(self.engine.as_mut());
                self.window.request_redraw();
            }
            _ => {
                let winit_event = winit::event::Event::WindowEvent {
                    window_id: _window_id,
                    event,
                };
                if let Some(input_event) = process_winit_event(&winit_event) {
                    self.game_app.input_event(self.engine.as_mut(), &input_event);
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

        let winit_event = winit::event::Event::DeviceEvent {
            device_id,
            event,
        };
        if let Some(input_event) = process_winit_event(&winit_event) {
            self.game_app.input_event(self.engine.as_mut(), &input_event);
        }
    }
}