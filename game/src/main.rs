mod blocks;
mod game;
mod player;
mod game_state;
mod voxel_world;
mod world_gen;
mod texture_loader;

fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut game = game::Game::default();
    event_loop.run_app(&mut game)
}