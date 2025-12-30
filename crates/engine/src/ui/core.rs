use glam::{Vec2, Vec4};

#[derive(Debug, Clone, Copy)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy)]
pub enum SizeMode {
    Fixed(Vec2),
    Relative(Vec2),
    FillParent,
    FitContent,
}

#[derive(Debug, Clone)]
pub struct Style {
    pub position: Vec2,
    pub size: Vec2,
    pub anchor: Anchor,
    pub size_mode: SizeMode,
    pub margin: Vec2,
    pub padding: Vec2,
    pub color: Vec4,
    pub visible: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::new(100.0, 50.0),
            anchor: Anchor::TopLeft,
            size_mode: SizeMode::Fixed(Vec2::new(100.0, 50.0)),
            margin: Vec2::ZERO,
            padding: Vec2::ZERO,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }
}

pub fn calculate_layout(style: &Style, parent_rect: Rect, content_size: Vec2) -> Rect {
    // Вычисляем размер
    let size = match style.size_mode {
        SizeMode::Fixed(s) => s,
        SizeMode::Relative(ratio) => Vec2::new(parent_rect.width * ratio.x, parent_rect.height * ratio.y),
        SizeMode::FillParent => Vec2::new(parent_rect.width - style.margin.x * 2.0, parent_rect.height - style.margin.y * 2.0),
        SizeMode::FitContent => content_size + style.padding * 2.0,
    };

    // Вычисляем позицию якоря в родителе
    let anchor_pos = match style.anchor {
        Anchor::TopLeft => Vec2::new(parent_rect.x, parent_rect.y),
        Anchor::TopCenter => Vec2::new(parent_rect.x + parent_rect.width * 0.5, parent_rect.y),
        Anchor::TopRight => Vec2::new(parent_rect.x + parent_rect.width, parent_rect.y),
        Anchor::CenterLeft => Vec2::new(parent_rect.x, parent_rect.y + parent_rect.height * 0.5),
        Anchor::Center => Vec2::new(parent_rect.x + parent_rect.width * 0.5, parent_rect.y + parent_rect.height * 0.5),
        Anchor::CenterRight => Vec2::new(parent_rect.x + parent_rect.width, parent_rect.y + parent_rect.height * 0.5),
        Anchor::BottomLeft => Vec2::new(parent_rect.x, parent_rect.y + parent_rect.height),
        Anchor::BottomCenter => Vec2::new(parent_rect.x + parent_rect.width * 0.5, parent_rect.y + parent_rect.height),
        Anchor::BottomRight => Vec2::new(parent_rect.x + parent_rect.width, parent_rect.y + parent_rect.height),
    };

    // Смещение элемента относительно якоря
    let element_anchor_offset = match style.anchor {
        Anchor::TopLeft => Vec2::ZERO,
        Anchor::TopCenter => Vec2::new(size.x * 0.5, 0.0),
        Anchor::TopRight => Vec2::new(size.x, 0.0),
        Anchor::CenterLeft => Vec2::new(0.0, size.y * 0.5),
        Anchor::Center => size * 0.5,
        Anchor::CenterRight => Vec2::new(size.x, size.y * 0.5),
        Anchor::BottomLeft => Vec2::new(0.0, size.y),
        Anchor::BottomCenter => Vec2::new(size.x * 0.5, size.y),
        Anchor::BottomRight => size,
    };

    let final_pos = anchor_pos + style.position + style.margin - element_anchor_offset;

    Rect::new(final_pos.x, final_pos.y, size.x, size.y)
}