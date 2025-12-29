use glam::{Vec2, Vec4};
use super::layout::UILayout;

#[derive(Debug, Clone)]
pub enum UIElement {
    Rect {
        id: String,
        layout: UILayout,
        color: Vec4,
        visible: bool,
    },
    Text {
        id: String,
        layout: UILayout,
        text: String,
        size: f32,
        color: Vec4,
        visible: bool,
    },
    Image {
        id: String,
        layout: UILayout,
        texture_path: String,
        visible: bool,
    },
}

impl UIElement {
    pub fn new_rect(id: &str, layout: UILayout, color: Vec4) -> Self {
        Self::Rect {
            id: id.to_string(),
            layout,
            color,
            visible: true,
        }
    }
    
    pub fn new_text(id: &str, layout: UILayout, text: &str, size: f32, color: Vec4) -> Self {
        Self::Text {
            id: id.to_string(),
            layout,
            text: text.to_string(),
            size,
            color,
            visible: true,
        }
    }
    
    pub fn new_image(id: &str, layout: UILayout, texture_path: &str) -> Self {
        Self::Image {
            id: id.to_string(),
            layout,
            texture_path: texture_path.to_string(),
            visible: true,
        }
    }
    
    pub fn get_id(&self) -> &str {
        match self {
            Self::Rect { id, .. } => id,
            Self::Text { id, .. } => id,
            Self::Image { id, .. } => id,
        }
    }
    
    pub fn get_layout(&self) -> &UILayout {
        match self {
            Self::Rect { layout, .. } => layout,
            Self::Text { layout, .. } => layout,
            Self::Image { layout, .. } => layout,
        }
    }
    
    pub fn is_visible(&self) -> bool {
        match self {
            Self::Rect { visible, .. } => *visible,
            Self::Text { visible, .. } => *visible,
            Self::Image { visible, .. } => *visible,
        }
    }
    
    pub fn set_visible(&mut self, visible: bool) {
        match self {
            Self::Rect { visible: v, .. } => *v = visible,
            Self::Text { visible: v, .. } => *v = visible,
            Self::Image { visible: v, .. } => *v = visible,
        }
    }
}