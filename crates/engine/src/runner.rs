use crate::{app::GameApp, app_runner::AppRunner};

pub fn run_app<T: GameApp + 'static>(game_app: T) -> Result<(), winit::error::EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app_runner = AppRunner::new(game_app);
    event_loop.run_app(&mut app_runner)
}