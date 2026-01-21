use std::sync::{Arc, Mutex};

use glam::{Vec2, Vec4};

use crate::engine::{UIRenderer, ui::{Container, LayoutType, Rect, SizeMode, Widget}};
#[derive(Clone)]
pub struct UI {
    pub root: Container,
}

impl UI {
    pub fn new() -> Self {
        Self {
            root: Container::new(LayoutType::Stack)
                .with_style(|s| {
                    s.color = Vec4::new(0.0, 0.0, 0.0, 0.0); // Прозрачный фон
                    s.size_mode = SizeMode::FillParent;
                }),
        }
    }

    pub fn add_widget(mut self, widget: impl Widget + 'static) -> Self {
        let widget_arc = Arc::new(Mutex::new(widget));
        self.root = self.root.add_child(widget_arc);
        self
    }

    pub fn render(&mut self, renderer: &mut UIRenderer) {
        let screen_rect = Rect::new(0.0, 0.0, renderer.screen_size.x, renderer.screen_size.y);
        self.root.render(renderer, screen_rect);
    }

    pub fn handle_click(&self, point: glam::Vec2) -> bool {
        self.root.handle_click(point)
    }
}

pub struct UIBuilder;

impl UIBuilder {
    pub fn new() -> UI {
        UI::new()
    }
}