use glam::{Vec2, Vec4};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum UIElementType {
    Container,
    Rect { color: Vec4 },
    Text { text: String, size: f32, color: Vec4 },
    Image { texture_path: String },
    Grid { columns: u32, rows: u32, spacing: Vec2 },
}

#[derive(Debug, Clone)]
pub struct UIElement {
    pub id: String,
    pub element_type: UIElementType,
    pub position: Vec2,
    pub size: Vec2,
    pub children: Vec<UIElement>,
    pub visible: bool,
}

impl UIElement {
    pub fn new_container(id: &str, position: Vec2, size: Vec2) -> Self {
        Self {
            id: id.to_string(),
            element_type: UIElementType::Container,
            position,
            size,
            children: Vec::new(),
            visible: true,
        }
    }
    
    pub fn new_rect(id: &str, position: Vec2, size: Vec2, color: Vec4) -> Self {
        Self {
            id: id.to_string(),
            element_type: UIElementType::Rect { color },
            position,
            size,
            children: Vec::new(),
            visible: true,
        }
    }
    
    pub fn new_text(id: &str, position: Vec2, text: &str, text_size: f32, color: Vec4) -> Self {
        Self {
            id: id.to_string(),
            element_type: UIElementType::Text { 
                text: text.to_string(), 
                size: text_size, 
                color 
            },
            position,
            size: Vec2::ZERO,
            children: Vec::new(),
            visible: true,
        }
    }
    
    pub fn new_image(id: &str, position: Vec2, size: Vec2, texture_path: &str) -> Self {
        Self {
            id: id.to_string(),
            element_type: UIElementType::Image { 
                texture_path: texture_path.to_string() 
            },
            position,
            size,
            children: Vec::new(),
            visible: true,
        }
    }
    
    pub fn new_grid(id: &str, position: Vec2, size: Vec2, columns: u32, rows: u32, spacing: Vec2) -> Self {
        Self {
            id: id.to_string(),
            element_type: UIElementType::Grid { columns, rows, spacing },
            position,
            size,
            children: Vec::new(),
            visible: true,
        }
    }
    
    pub fn add_child(&mut self, child: UIElement) {
        self.children.push(child);
    }
    
    pub fn render(&self, engine: &mut voxel_engine::Engine, parent_pos: Vec2, screen_size: Vec2) {
        if !self.visible { return; }
        
        let abs_pos = parent_pos + self.position;
        let norm_pos = Vec2::new(abs_pos.x / screen_size.x, abs_pos.y / screen_size.y);
        let norm_size = Vec2::new(self.size.x / screen_size.x, self.size.y / screen_size.y);
        
        match &self.element_type {
            UIElementType::Container => {
                // Контейнер сам по себе ничего не рендерит
            }
            UIElementType::Rect { color } => {
                engine.add_ui_rect(norm_pos, norm_size, *color);
            }
            UIElementType::Text { text, size, color } => {
                engine.add_ui_text(text, norm_pos, *size, *color);
            }
            UIElementType::Image { texture_path } => {
                if engine.get_ui_texture_id(texture_path).is_none() {
                    engine.load_ui_texture(texture_path);
                }
                if let Some(texture_id) = engine.get_ui_texture_id(texture_path) {
                    engine.add_ui_image(norm_pos, norm_size, texture_id);
                }
            }
            UIElementType::Grid { columns, rows, spacing } => {
                let cell_width = (self.size.x - spacing.x * (*columns - 1) as f32) / *columns as f32;
                let cell_height = (self.size.y - spacing.y * (*rows - 1) as f32) / *rows as f32;
                
                for (i, child) in self.children.iter().enumerate() {
                    let col = i as u32 % columns;
                    let row = i as u32 / columns;
                    
                    let cell_pos = Vec2::new(
                        col as f32 * (cell_width + spacing.x),
                        row as f32 * (cell_height + spacing.y)
                    );
                    
                    let mut cell_element = child.clone();
                    cell_element.position = cell_pos;
                    cell_element.render(engine, abs_pos, screen_size);
                }
                return;
            }
        }
        
        // Рендерим дочерние элементы
        for child in &self.children {
            child.render(engine, abs_pos, screen_size);
        }
    }
    
    pub fn find_element(&self, id: &str, parent_pos: Vec2) -> Option<(Vec2, Vec2)> {
        let abs_pos = parent_pos + self.position;
        
        if self.id == id {
            return Some((abs_pos, self.size));
        }
        
        // Поиск в дочерних элементах
        for child in &self.children {
            if let Some(result) = child.find_element(id, abs_pos) {
                return Some(result);
            }
        }
        
        None
    }
    
    pub fn hit_test(&self, point: Vec2, parent_pos: Vec2) -> Option<String> {
        let abs_pos = parent_pos + self.position;
        
        // Проверяем попадание в текущий элемент
        if point.x >= abs_pos.x && point.x <= abs_pos.x + self.size.x &&
           point.y >= abs_pos.y && point.y <= abs_pos.y + self.size.y {
            
            // Сначала проверяем дочерние элементы (они сверху)
            for child in &self.children {
                if let Some(hit_id) = child.hit_test(point, abs_pos) {
                    return Some(hit_id);
                }
            }
            
            // Если дочерние не попали, возвращаем текущий элемент
            return Some(self.id.clone());
        }
        
        None
    }
}