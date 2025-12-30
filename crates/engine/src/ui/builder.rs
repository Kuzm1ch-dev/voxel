use glam::Vec4;
use crate::ui::{core::*, container::Container, widgets::Widget, LayoutType};

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

    pub fn add_widget(mut self, widget: Box<dyn Widget>) -> Self {
        self.root = self.root.add_child(widget);
        self
    }

    pub fn render(&self, renderer: &mut crate::UIRenderer) {
        let screen_rect = Rect::new(0.0, 0.0, renderer.screen_size.x, renderer.screen_size.y);
        self.root.render(renderer, screen_rect);
    }

    pub fn handle_click(&self, point: glam::Vec2) -> bool {
        let screen_rect = Rect::new(0.0, 0.0, 800.0, 600.0); // TODO: получать размер экрана
        self.root.handle_click(point, screen_rect)
    }
}

pub struct UIBuilder;

impl UIBuilder {
    pub fn new() -> UI {
        UI::new()
    }
}