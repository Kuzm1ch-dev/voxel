use std::sync::Arc;

use crate::engine::{Engine, InputEvent};

pub trait GameApp {
    fn ready(&mut self, engine: &mut Engine);
    fn update(&mut self, engine: &mut Engine, delta_time: f32);
    fn input_event(&mut self, engine: &mut Engine, event: &InputEvent);
    fn render(&mut self, engine: &mut Engine);
    fn resize(&mut self, engine: &mut Engine, new_size: winit::dpi::PhysicalSize<u32>);
}