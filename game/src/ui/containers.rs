use glam::{Vec2, Vec4};
use super::layout::{UILayout, LayoutDirection, Anchor, SizeMode};
use super::elements::UIElement;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UIContainer {
    pub id: String,
    pub layout: UILayout,
    pub children: Vec<String>,
    pub visible: bool,
    pub background_color: Option<Vec4>,
}

#[derive(Debug, Clone)]
pub struct UIGridContainer {
    pub id: String,
    pub layout: UILayout,
    pub columns: u32,
    pub rows: u32,
    pub cell_spacing: Vec2,
    pub children: Vec<UIElement>,
    pub visible: bool,
    pub background_color: Option<Vec4>,
}

#[derive(Debug, Clone)]
pub struct UIFlexContainer {
    pub id: String,
    pub layout: UILayout,
    pub direction: LayoutDirection,
    pub spacing: f32,
    pub children: Vec<String>,
    pub visible: bool,
    pub background_color: Option<Vec4>,
}

pub struct UIContainerSystem {
    containers: HashMap<String, UIContainer>,
    grid_containers: HashMap<String, UIGridContainer>,
    flex_containers: HashMap<String, UIFlexContainer>,
    screen_size: Vec2,
}

impl UIContainerSystem {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
            grid_containers: HashMap::new(),
            flex_containers: HashMap::new(),
            screen_size: Vec2::new(800.0, 600.0), // Дефолтный размер
        }
    }
    
    pub fn set_screen_size(&mut self, size: Vec2) {
        self.screen_size = size;
    }
    
    // Обычный контейнер
    pub fn add_container(&mut self, id: &str, layout: UILayout) -> &mut UIContainer {
        let container = UIContainer {
            id: id.to_string(),
            layout,
            children: Vec::new(),
            visible: true,
            background_color: None,
        };
        self.containers.insert(id.to_string(), container);
        self.containers.get_mut(id).unwrap()
    }
    
    // Сеточный контейнер (для инвентаря)
    pub fn add_grid_container(&mut self, id: &str, layout: UILayout, columns: u32, rows: u32) -> &mut UIGridContainer {
        let container = UIGridContainer {
            id: id.to_string(),
            layout,
            columns,
            rows,
            cell_spacing: Vec2::new(5.0, 5.0),
            children: Vec::new(),
            visible: true,
            background_color: None,
        };
        self.grid_containers.insert(id.to_string(), container);
        self.grid_containers.get_mut(id).unwrap()
    }
    
    // Flex контейнер (ряд/столбик)
    pub fn add_flex_container(&mut self, id: &str, layout: UILayout, direction: LayoutDirection) -> &mut UIFlexContainer {
        let container = UIFlexContainer {
            id: id.to_string(),
            layout,
            direction,
            spacing: 10.0,
            children: Vec::new(),
            visible: true,
            background_color: None,
        };
        self.flex_containers.insert(id.to_string(), container);
        self.flex_containers.get_mut(id).unwrap()
    }
    
    // Вычисляет позицию и размер контейнера
    pub fn calculate_container_bounds(&self, container_id: &str, parent_pos: Vec2, parent_size: Vec2) -> (Vec2, Vec2) {
        if let Some(container) = self.containers.get(container_id) {
            let size = container.layout.calculate_size(parent_size, Vec2::ZERO);
            let pos = container.layout.calculate_position(parent_pos, parent_size, size);
            return (pos, size);
        }
        
        if let Some(container) = self.grid_containers.get(container_id) {
            let size = container.layout.calculate_size(parent_size, Vec2::ZERO);
            let pos = container.layout.calculate_position(parent_pos, parent_size, size);
            return (pos, size);
        }
        
        if let Some(container) = self.flex_containers.get(container_id) {
            let size = container.layout.calculate_size(parent_size, Vec2::ZERO);
            let pos = container.layout.calculate_position(parent_pos, parent_size, size);
            return (pos, size);
        }
        
        (Vec2::ZERO, Vec2::ZERO)
    }
    
    // Вычисляет позицию элемента в сеточном контейнере
    pub fn calculate_grid_cell_position(&self, container_id: &str, cell_index: u32, parent_pos: Vec2, parent_size: Vec2) -> Option<(Vec2, Vec2)> {
        let container = self.grid_containers.get(container_id)?;
        let container_size = container.layout.calculate_size(parent_size, Vec2::ZERO);
        let container_pos = container.layout.calculate_position(parent_pos, parent_size, container_size);
        
        let col = cell_index % container.columns;
        let row = cell_index / container.columns;
        
        let cell_width = (container_size.x - container.cell_spacing.x * (container.columns - 1) as f32) / container.columns as f32;
        let cell_height = (container_size.y - container.cell_spacing.y * (container.rows - 1) as f32) / container.rows as f32;
        
        let cell_pos = container_pos + Vec2::new(
            col as f32 * (cell_width + container.cell_spacing.x),
            row as f32 * (cell_height + container.cell_spacing.y)
        );
        
        Some((cell_pos, Vec2::new(cell_width, cell_height)))
    }
    
    // Вычисляет позицию элемента в flex контейнере
    pub fn calculate_flex_item_position(&self, container_id: &str, item_index: u32, item_count: u32) -> Option<(Vec2, Vec2)> {
        let container = self.flex_containers.get(container_id)?;
        let (container_pos, container_size) = self.calculate_container_bounds(container_id, Vec2::ZERO, self.screen_size);
        
        match container.direction {
            LayoutDirection::Horizontal => {
                let item_width = (container_size.x - container.spacing * (item_count - 1) as f32) / item_count as f32;
                let item_pos = container_pos + Vec2::new(
                    item_index as f32 * (item_width + container.spacing),
                    0.0
                );
                Some((item_pos, Vec2::new(item_width, container_size.y)))
            }
            LayoutDirection::Vertical => {
                let item_height = (container_size.y - container.spacing * (item_count - 1) as f32) / item_count as f32;
                let item_pos = container_pos + Vec2::new(
                    0.0,
                    item_index as f32 * (item_height + container.spacing)
                );
                Some((item_pos, Vec2::new(container_size.x, item_height)))
            }
        }
    }
    
    // Рендерит обычный контейнер
    pub fn render_container(&self, container_id: &str, engine: &mut voxel_engine::Engine) {
        if let Some(container) = self.containers.get(container_id) {
            if !container.visible { return; }
            
            let (pos, size) = self.calculate_container_bounds(container_id, Vec2::ZERO, self.screen_size);
            let norm_pos = Vec2::new(pos.x / self.screen_size.x, pos.y / self.screen_size.y);
            let norm_size = Vec2::new(size.x / self.screen_size.x, size.y / self.screen_size.y);
            
            if let Some(bg_color) = container.background_color {
                engine.add_ui_rect(norm_pos, norm_size, bg_color);
            } else {
                engine.add_ui_rect(norm_pos, norm_size, crate::ui::colors::DARK_GRAY);
            }
            
            // Заголовок и кнопка закрытия для инвентаря
            if container_id == "inventory_main" {
                engine.add_ui_text("INVENTORY", norm_pos + Vec2::new(0.02, 0.02), 2.0, crate::ui::colors::WHITE);
                engine.add_ui_text("PRESS I TO CLOSE", norm_pos + Vec2::new(0.02, norm_size.y - 0.05), 1.0, crate::ui::colors::YELLOW);
                
                engine.add_ui_rect(norm_pos + Vec2::new(norm_size.x - 0.08, 0.01), Vec2::new(0.06, 0.04), crate::ui::colors::GRAY);
                engine.add_ui_text("X", norm_pos + Vec2::new(norm_size.x - 0.06, 0.02), 1.0, crate::ui::colors::WHITE);
            }
        }
    }
    
    // Добавляет элемент в ячейку сетки
    pub fn add_grid_cell_element(&mut self, grid_id: &str, cell_index: u32, element: UIElement) {
        if let Some(grid) = self.grid_containers.get_mut(grid_id) {
            // Расширяем массив если нужно
            while grid.children.len() <= cell_index as usize {
                grid.children.push(UIElement::new_rect(
                    &format!("{}_empty_{}", grid_id, grid.children.len()),
                    UILayout::new(),
                    Vec4::ZERO
                ));
            }
            grid.children[cell_index as usize] = element;
        }
    }
    
    // Рендерит сеточный контейнер с дочерними элементами
    pub fn render_grid_container(&self, container_id: &str, engine: &mut voxel_engine::Engine) {
        if let Some(container) = self.grid_containers.get(container_id) {
            if !container.visible { return; }
            
            for (i, element) in container.children.iter().enumerate() {
                if !element.is_visible() { continue; }
                
                if let Some((cell_pos, cell_size)) = self.calculate_grid_cell_position(container_id, i as u32, Vec2::ZERO, self.screen_size) {
                    let norm_pos = Vec2::new(cell_pos.x / self.screen_size.x, cell_pos.y / self.screen_size.y);
                    let norm_size = Vec2::new(cell_size.x / self.screen_size.x, cell_size.y / self.screen_size.y);
                    
                    self.render_element(element, norm_pos, norm_size, engine);
                }
            }
        }
    }
    
    // Рендерит отдельный UI элемент
    fn render_element(&self, element: &UIElement, parent_pos: Vec2, parent_size: Vec2, engine: &mut voxel_engine::Engine) {
        let size = element.get_layout().calculate_size(parent_size, Vec2::ZERO);
        let pos = element.get_layout().calculate_position(parent_pos, parent_size, size);
        
        match element {
            UIElement::Rect { color, .. } => {
                engine.add_ui_rect(pos, size, *color);
            }
            UIElement::Text { text, size: text_size, color, .. } => {
                engine.add_ui_text(text, pos, *text_size, *color);
            }
            UIElement::Image { texture_path, .. } => {
                if engine.get_ui_texture_id(texture_path).is_none() {
                    engine.load_ui_texture(texture_path);
                }
                if let Some(texture_id) = engine.get_ui_texture_id(texture_path) {
                    engine.add_ui_image(pos, size, texture_id);
                }
            }
        }
    }
}