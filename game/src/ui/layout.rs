use glam::Vec2;

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
    Fixed(Vec2),           // Фиксированный размер
    Relative(Vec2),        // Относительный размер (0.0-1.0)
    FillParent,           // Заполнить родителя
    FitContent,           // По размеру содержимого
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct UILayout {
    pub anchor: Anchor,
    pub offset: Vec2,
    pub size_mode: SizeMode,
    pub margin: Vec2,
    pub padding: Vec2,
}

impl Default for UILayout {
    fn default() -> Self {
        Self {
            anchor: Anchor::TopLeft,
            offset: Vec2::ZERO,
            size_mode: SizeMode::Fixed(Vec2::new(100.0, 50.0)),
            margin: Vec2::ZERO,
            padding: Vec2::ZERO,
        }
    }
}

impl UILayout {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
    
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn with_size(mut self, size_mode: SizeMode) -> Self {
        self.size_mode = size_mode;
        self
    }
    
    pub fn with_margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }
    
    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = padding;
        self
    }
    
    // Вычисляет финальную позицию элемента
    pub fn calculate_position(&self, parent_pos: Vec2, parent_size: Vec2, element_size: Vec2) -> Vec2 {
        let anchor_pos = self.get_anchor_position(parent_pos, parent_size);
        let element_anchor_offset = self.get_element_anchor_offset(element_size);
        
        anchor_pos + self.offset + self.margin - element_anchor_offset
    }
    
    // Вычисляет финальный размер элемента
    pub fn calculate_size(&self, parent_size: Vec2, content_size: Vec2) -> Vec2 {
        match self.size_mode {
            SizeMode::Fixed(size) => size,
            SizeMode::Relative(ratio) => parent_size * ratio,
            SizeMode::FillParent => parent_size - self.margin * 2.0 - self.padding * 2.0,
            SizeMode::FitContent => content_size + self.padding * 2.0,
        }
    }
    
    fn get_anchor_position(&self, parent_pos: Vec2, parent_size: Vec2) -> Vec2 {
        match self.anchor {
            Anchor::TopLeft => parent_pos,
            Anchor::TopCenter => parent_pos + Vec2::new(parent_size.x * 0.5, 0.0),
            Anchor::TopRight => parent_pos + Vec2::new(parent_size.x, 0.0),
            Anchor::CenterLeft => parent_pos + Vec2::new(0.0, parent_size.y * 0.5),
            Anchor::Center => parent_pos + parent_size * 0.5,
            Anchor::CenterRight => parent_pos + Vec2::new(parent_size.x, parent_size.y * 0.5),
            Anchor::BottomLeft => parent_pos + Vec2::new(0.0, parent_size.y),
            Anchor::BottomCenter => parent_pos + Vec2::new(parent_size.x * 0.5, parent_size.y),
            Anchor::BottomRight => parent_pos + parent_size,
        }
    }
    
    fn get_element_anchor_offset(&self, element_size: Vec2) -> Vec2 {
        match self.anchor {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::TopCenter => Vec2::new(element_size.x * 0.5, 0.0),
            Anchor::TopRight => Vec2::new(element_size.x, 0.0),
            Anchor::CenterLeft => Vec2::new(0.0, element_size.y * 0.5),
            Anchor::Center => element_size * 0.5,
            Anchor::CenterRight => Vec2::new(element_size.x, element_size.y * 0.5),
            Anchor::BottomLeft => Vec2::new(0.0, element_size.y),
            Anchor::BottomCenter => Vec2::new(element_size.x * 0.5, element_size.y),
            Anchor::BottomRight => element_size,
        }
    }
}