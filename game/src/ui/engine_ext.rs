use voxel_engine::Engine;
use crate::ui::widget::UIElement;
use glam::Vec2;

pub trait EngineUIExt {
    fn render_ui_element(&mut self, element: &UIElement, screen_size: Vec2);
}

impl EngineUIExt for Engine<'_> {
    fn render_ui_element(&mut self, element: &UIElement, screen_size: Vec2) {
        element.render(self, Vec2::ZERO, screen_size);
    }
}