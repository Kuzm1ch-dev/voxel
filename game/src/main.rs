mod blocks;
mod common;
mod game;
mod player;
mod game_state;
mod voxel_world;
mod world_gen;
// mod ui;
mod systems;
mod utils;

fn main() -> Result<(), winit::error::EventLoopError> {
    // Устанавливаем рабочую директорию в папку game
    if let Err(_) = std::env::set_current_dir("game") {
        // Если мы уже в папке game, ничего не делаем
    }
    
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut game = game::Game::default();
    event_loop.run_app(&mut game)
}