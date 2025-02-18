use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use crate::render::renderer::Renderer;


#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'window>>,
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("wgpu winit example");
            // use Arc.
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            // let wgpu_ctx = WgpuCtx::new(window.clone());
            // self.wgpu_ctx = Some(wgpu_ctx);
            let renderer = Renderer::new(window.clone());
            self.renderer = Some(renderer);
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
                if let (Some(renderer), Some(window)) =
                    (self.renderer.as_mut(), self.window.as_ref())
                {
                    renderer.resize(new_size);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.render();
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.keyboard_input(device_id, event, is_synthetic);
                }
            }
            _ => (),
        }
    }

    fn device_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            device_id: winit::event::DeviceId,
            event: winit::event::DeviceEvent,
        ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.process_mouse(delta.0 as f32, delta.1 as f32);
                }
            }
            _ => ()
        }
    }
}
