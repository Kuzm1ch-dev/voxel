use std::sync::Arc;

use winit::window::Window;

use crate::engine::{GameApp, app_runner::AppRunner};


pub fn run_app<T: GameApp + 'static>(game_app: T) -> Result<(), winit::error::EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let win_attr = Window::default_attributes().with_title("Title");
    let window: Arc<Window> = Arc::new(
        event_loop
            .create_window(win_attr)
            .expect("Failed to create window"),
    );
    let mut app_runner = AppRunner::new(game_app, window);
    event_loop.run_app(&mut app_runner)
}