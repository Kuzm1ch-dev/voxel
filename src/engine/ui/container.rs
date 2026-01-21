use std::sync::{Arc, Mutex};

use glam::Vec2;

use crate::engine::{UIRenderer, ui::{Button, Panel, Rect, Style, Text, Widget, calculate_layout}};

#[derive(Debug, Clone, Copy)]
pub enum LayoutType {
    Vertical { spacing: f32 },
    Horizontal { spacing: f32 },
    Grid { columns: usize, spacing: f32 },
    Stack, // Элементы накладываются друг на друга
}
#[derive(Clone)]
pub struct Container {
    pub style: Style,
    pub layout: LayoutType,
    pub children: Vec<Arc<Mutex<dyn Widget>>>,
}

impl Container {
    pub fn new(layout: LayoutType) -> Self {
        Self {
            style: Style::default(),
            layout,
            children: Vec::new(),
        }
    }

    pub fn with_style<F>(mut self, f: F) -> Self 
    where F: FnOnce(&mut Style) {
        f(&mut self.style);
        self
    }

    pub fn add_child(mut self, child: Arc<Mutex<dyn Widget>>) -> Self {
        self.children.push(child);
        self
    }

    pub fn add_text(self, text: Text) -> Self {
        self.add_child(Arc::new(Mutex::new(text)))
    }

    pub fn add_panel(self, panel: Panel) -> Self {
        self.add_child(Arc::new(Mutex::new(panel)))
    }

    pub fn add_button(self, button: Button) -> Self {
        self.add_child(Arc::new(Mutex::new(button)))
    }

    pub fn add_container(self, container: Container) -> Self {
        self.add_child(Arc::new(Mutex::new(container)))
    }
}

impl Widget for Container {
    fn style(&self) -> &Style { &self.style }
    fn style_mut(&mut self) -> &mut Style { &mut self.style }

    fn render(&mut self, renderer: &mut UIRenderer, rect: Rect) {
        if !self.style.visible { return; }

        let layout_rect = calculate_layout(&self.style, rect, Vec2::ZERO);
        if self.style.color.w > 0.0 {
            renderer.render_rect(Vec2::new(layout_rect.x, layout_rect.y), Vec2::new(layout_rect.width, layout_rect.height), self.style.color);
        }

        let content_rect = Rect::new(
            layout_rect.x + self.style.padding.x,
            layout_rect.y + self.style.padding.y,
            layout_rect.width - self.style.padding.x * 2.0,
            layout_rect.height - self.style.padding.y * 2.0,
        );

        self.render_children(renderer, content_rect);
    }

    fn handle_click(&self, point: Vec2) -> bool {
        if !self.style.visible { return false; }

        for child in self.children.iter().rev() {
            let mut lock_child = child.lock().unwrap();
            if lock_child.handle_click(point) {
                return true;
            }
        }
        false
    }
}

impl Container {
    fn render_children(&mut self, renderer: &mut UIRenderer, content_rect: Rect) {
        match self.layout {
            LayoutType::Vertical { spacing } => {
                let mut current_y = content_rect.y;
                for child in &mut self.children {
                    let mut lock_child = child.lock().unwrap();
                    let child_height = if lock_child.style().size.y > 0.0 { lock_child.style().size.y } else { 40.0 };
                    let positioned_rect = Rect::new(content_rect.x, current_y, content_rect.width, child_height);
                    lock_child.render(renderer, positioned_rect);
                    current_y += child_height + spacing;
                }
            },
            LayoutType::Horizontal { spacing } => {
                let mut current_x = content_rect.x;
                for child in &mut self.children {
                    let mut lock_child = child.lock().unwrap();
                    let positioned_rect = Rect::new(current_x, content_rect.y, 100.0, content_rect.height);
                    lock_child.render(renderer, positioned_rect);
                    current_x += 100.0 + spacing;
                }
            },
            LayoutType::Grid { columns, spacing } => {
                for (i, child) in self.children.iter_mut().enumerate() {
                    let row = i / columns;
                    let col = i % columns;
                    let cell_width = (content_rect.width - spacing * (columns - 1) as f32) / columns as f32;
                    let cell_height = 40.0;
                    let mut lock_child = child.lock().unwrap();
                    let cell_rect = Rect::new(
                        content_rect.x + col as f32 * (cell_width + spacing),
                        content_rect.y + row as f32 * (cell_height + spacing),
                        cell_width,
                        cell_height
                    );
                    
                    lock_child.render(renderer, cell_rect);
                }
            },
            LayoutType::Stack => {
                for child in &mut self.children {
                    let mut lock_child = child.lock().unwrap();
                    lock_child.render(renderer, content_rect);
                }
            }
        }
    }

}