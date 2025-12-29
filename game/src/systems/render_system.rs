use voxel_engine::Engine;
use crate::{game_state::VoxelGameState, systems::ui_system::UISystem};
pub struct RenderSystem;

impl RenderSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, engine: &mut Engine, game_state: &mut VoxelGameState, ui_system: &mut UISystem) -> Result<(), wgpu::SurfaceError> {
        // Update camera
        let player = &game_state.player;
        engine.update_camera(
            player.get_camera_position(),
            player.get_camera_target(),
            player.get_camera_up()
        );
        
        // Render UI
        ui_system.render(engine, player.get_camera_position(), game_state);
        
        // Render world
        game_state.world.render(engine);
        
        // Final render
        engine.render()
    }
}