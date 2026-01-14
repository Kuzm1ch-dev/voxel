use std::sync::Arc;
use crate::input::InputEvent;

pub trait GameApp {
    fn ready(&mut self, engine: &mut crate::Engine);
    fn update(&mut self, engine: &mut crate::Engine, delta_time: f32);
    fn input_event(&mut self, engine: &mut crate::Engine, event: &InputEvent);
    fn render(&mut self, engine: &mut crate::Engine);
    fn resize(&mut self, engine: &mut crate::Engine, new_size: winit::dpi::PhysicalSize<u32>);
}