use glam::{Vec2, Vec4};
use crate::ui::core::*;

pub trait Widget {
    fn style(&self) -> &Style;
    fn style_mut(&mut self) -> &mut Style;
    fn render(&mut self, renderer: &mut crate::UIRenderer, rect: Rect);
    fn handle_click(&self, point: Vec2) -> bool { false }
    fn content_size(&self) -> Vec2 { Vec2::ZERO }
}

pub struct Text {
    pub style: Style,
    pub text: String,
    pub scale: f32,
}

impl Text {
    pub fn new(text: &str) -> Self {
        Self {
            style: Style::default(),
            text: text.to_string(),
            scale: 1.0,
        }
    }

    pub fn with_style<F>(mut self, f: F) -> Self 
    where F: FnOnce(&mut Style) {
        f(&mut self.style);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

impl Widget for Text {
    fn style(&self) -> &Style { &self.style }
    fn style_mut(&mut self) -> &mut Style { &mut self.style }

    fn render(&mut self, renderer: &mut crate::UIRenderer, rect: Rect) {
        if !self.style.visible || self.style.color.w <= 0.0 { return; }
        let layout_rect = calculate_layout(&self.style, rect, self.content_size());
        renderer.render_text(&self.text, Vec2::new(layout_rect.x, layout_rect.y), self.scale, self.style.color);
    }

    fn content_size(&self) -> Vec2 {
        Vec2::new(self.text.len() as f32 * 8.0 * self.scale, 8.0 * self.scale)
    }
}

pub struct Panel {
    pub style: Style,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
        }
    }

    pub fn with_style<F>(mut self, f: F) -> Self 
    where F: FnOnce(&mut Style) {
        f(&mut self.style);
        self
    }
}

impl Widget for Panel {
    fn style(&self) -> &Style { &self.style }
    fn style_mut(&mut self) -> &mut Style { &mut self.style }

    fn render(&mut self, renderer: &mut crate::UIRenderer, rect: Rect) {
        if !self.style.visible { return; }
        let layout_rect = calculate_layout(&self.style, rect, Vec2::ZERO);
        if self.style.color.w > 0.0 {
            renderer.render_rect(Vec2::new(layout_rect.x, layout_rect.y), Vec2::new(layout_rect.width, layout_rect.height), self.style.color);
        }
    }
}

pub struct Button {
    pub style: Style,
    pub text: String,
    pub text_color: Vec4,
    pub scale: f32,
    pub on_click: Option<Box<dyn Fn()>>,
    pub rect: Rect,
}

impl Button {
    pub fn new(text: &str) -> Self {
        Self {
            style: Style {
                color: Vec4::new(0.3, 0.3, 0.3, 1.0),
                ..Style::default()
            },
            text: text.to_string(),
            text_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            scale: 1.0,
            on_click: None,
            rect: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn with_style<F>(mut self, f: F) -> Self 
    where F: FnOnce(&mut Style) {
        f(&mut self.style);
        self
    }

    pub fn with_text_color(mut self, color: Vec4) -> Self {
        self.text_color = color;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn on_click<F>(mut self, callback: F) -> Self 
    where F: Fn() + 'static {
        self.on_click = Some(Box::new(callback));
        self
    }
}

impl Widget for Button {
    fn style(&self) -> &Style { &self.style }
    fn style_mut(&mut self) -> &mut Style { &mut self.style }

    fn render(&mut self, renderer: &mut crate::UIRenderer, rect: Rect) {
        if !self.style.visible { return; }
        
        let layout_rect = calculate_layout(&self.style, rect, self.content_size());
        self.rect = layout_rect;
        
        renderer.render_rect(Vec2::new(layout_rect.x, layout_rect.y), Vec2::new(layout_rect.width, layout_rect.height), self.style.color);
        
        let text_size = Vec2::new(self.text.len() as f32 * 8.0 * self.scale, 8.0 * self.scale);
        let text_pos = Vec2::new(
            layout_rect.x + (layout_rect.width - text_size.x) * 0.5,
            layout_rect.y + (layout_rect.height - text_size.y) * 0.5
        );
        renderer.render_text(&self.text, text_pos, self.scale, self.text_color);
    }

    fn handle_click(&self, point: Vec2) -> bool {
        if !self.style.visible { return false; }
        if self.rect.contains(point) {
            if let Some(ref callback) = self.on_click {
                callback();
            }
            return true;
        }
        false
    }

    fn content_size(&self) -> Vec2 {
        Vec2::new(self.text.len() as f32 * 8.0 * self.scale + 20.0, 8.0 * self.scale + 10.0)
    }
}