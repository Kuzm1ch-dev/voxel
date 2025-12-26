use glam::{IVec3, Vec3};
use winit::event::{DeviceEvent, KeyEvent, MouseButton, ElementState};
use crate::world::block::BlockType;
use crate::world::chunk::Chunk;

/// Трейт для генерации мира
pub trait WorldGenerator {
    fn generate_chunk(&self, chunk_pos: IVec3) -> Chunk;
    fn get_block_at(&self, world_pos: IVec3) -> Option<BlockType>;
}

/// Трейт для обработки ввода
pub trait InputHandler {
    fn handle_keyboard(&mut self, event: KeyEvent) -> bool;
    fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) -> bool;
    fn handle_mouse_motion(&mut self, delta_x: f32, delta_y: f32) -> bool;
    fn handle_device_event(&mut self, event: DeviceEvent) -> bool;
}

/// Трейт для игровых сущностей
pub trait Entity {
    fn update(&mut self, dt: f32);
    fn get_position(&self) -> Vec3;
    fn set_position(&mut self, position: Vec3);
}

/// Трейт для игрока
pub trait Player: Entity {
    fn get_camera_position(&self) -> Vec3;
    fn get_camera_target(&self) -> Vec3;
    fn get_camera_up(&self) -> Vec3;
    fn move_forward(&mut self, amount: f32);
    fn move_backward(&mut self, amount: f32);
    fn move_left(&mut self, amount: f32);
    fn move_right(&mut self, amount: f32);
    fn look(&mut self, yaw: f32, pitch: f32);
}

/// Трейт для игрового состояния
pub trait GameState {
    fn update(&mut self, dt: f32);
    fn get_player(&self) -> &dyn Player;
    fn get_player_mut(&mut self) -> &mut dyn Player;
    fn get_world_generator(&self) -> &dyn WorldGenerator;
    fn get_input_handler(&mut self) -> &mut dyn InputHandler;
}