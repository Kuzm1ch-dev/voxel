mod blocks;
mod common;
mod game;
mod player;
mod game_state;
mod world;
mod systems;
mod utils;
mod modding;

fn main() -> Result<(), winit::error::EventLoopError> {
    // Устанавливаем рабочую директорию в папку game
    if let Err(_) = std::env::set_current_dir("game") {
        // Если мы уже в папке game, ничего не делаем
    }
    
    let game = game::Game::default();
    voxel_engine::run_app(game)
}