use glam::{Vec2, Vec4};
use voxel_engine::Engine;
use crate::ui::colors::*;
use crate::common::block_registry::BlockRegistry;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UIPanel {
    pub id: String,
    pub pos: Vec2,
    pub size: Vec2,
    pub bg_color: Vec4,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct UIText {
    pub id: String,
    pub pos: Vec2,
    pub text: String,
    pub scale: f32,
    pub color: Vec4,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct UIButton {
    pub panel: UIPanel,
    pub text: UIText,
    pub clickable: bool,
}

#[derive(Debug, Clone)]
pub struct UIImage {
    pub id: String,
    pub pos: Vec2,
    pub size: Vec2,
    pub texture_path: String,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct UIBlockDisplay {
    pub id: String,
    pub pos: Vec2,
    pub size: Vec2,
    pub block_id: String,
    pub visible: bool,
}

pub struct UIManager {
    panels: HashMap<String, UIPanel>,
    texts: HashMap<String, UIText>,
    buttons: HashMap<String, UIButton>,
    images: HashMap<String, UIImage>,
    block_displays: HashMap<String, UIBlockDisplay>,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            texts: HashMap::new(),
            buttons: HashMap::new(),
            images: HashMap::new(),
            block_displays: HashMap::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.panels.clear();
        self.texts.clear();
        self.buttons.clear();
        self.images.clear();
        self.block_displays.clear();
    }
    
    pub fn add_panel(&mut self, id: &str, pos: Vec2, size: Vec2, bg_color: Vec4) {
        self.panels.insert(id.to_string(), UIPanel {
            id: id.to_string(),
            pos,
            size,
            bg_color,
            visible: true,
        });
    }
    
    pub fn add_text(&mut self, id: &str, pos: Vec2, text: &str, scale: f32, color: Vec4) {
        self.texts.insert(id.to_string(), UIText {
            id: id.to_string(),
            pos,
            text: text.to_string(),
            scale,
            color,
            visible: true,
        });
    }
    
    pub fn add_button(&mut self, id: &str, pos: Vec2, size: Vec2, text: &str, bg_color: Vec4, text_color: Vec4) {
        let panel = UIPanel {
            id: format!("{}_panel", id),
            pos,
            size,
            bg_color,
            visible: true,
        };
        
        let text_pos = Vec2::new(pos.x + size.x * 0.1, pos.y + size.y * 0.3);
        let ui_text = UIText {
            id: format!("{}_text", id),
            pos: text_pos,
            text: text.to_string(),
            scale: 1.0,
            color: text_color,
            visible: true,
        };
        
        self.buttons.insert(id.to_string(), UIButton {
            panel,
            text: ui_text,
            clickable: true,
        });
    }
    
    pub fn add_image(&mut self, id: &str, pos: Vec2, size: Vec2, texture_path: &str) {
        self.images.insert(id.to_string(), UIImage {
            id: id.to_string(),
            pos,
            size,
            texture_path: texture_path.to_string(),
            visible: true,
        });
    }
    
    pub fn add_block_display(&mut self, id: &str, pos: Vec2, size: Vec2, block_id: &str) {
        self.block_displays.insert(id.to_string(), UIBlockDisplay {
            id: id.to_string(),
            pos,
            size,
            block_id: block_id.to_string(),
            visible: true,
        });
    }
    
    pub fn render(&self, engine: &mut Engine, registry: &BlockRegistry) {
        engine.clear_ui();
        
        // Render panels
        for panel in self.panels.values() {
            if panel.visible {
                engine.add_ui_rect(panel.pos, panel.size, panel.bg_color);
            }
        }
        
        // Render buttons (panel + text)
        for button in self.buttons.values() {
            if button.panel.visible && button.clickable {
                engine.add_ui_rect(button.panel.pos, button.panel.size, button.panel.bg_color);
                if button.text.visible {
                    engine.add_ui_text(&button.text.text, button.text.pos, button.text.scale, button.text.color);
                }
            }
        }
        
        // Render texts
        for text in self.texts.values() {
            if text.visible {
                engine.add_ui_text(&text.text, text.pos, text.scale, text.color);
            }
        }
        
        // Render images
        for image in self.images.values() {
            if image.visible {
                if let Some(texture_id) = engine.get_ui_texture_id(&image.texture_path) {
                    engine.add_ui_image(image.pos, image.size, texture_id);
                }
            }
        }
        
        // Render block displays
        for block_display in self.block_displays.values() {
            if block_display.visible {
              
                if let Some(block) = registry.get_block(&block_display.block_id) {
                    let texture_path = block.get_texture_path();
                    
                    if !texture_path.is_empty() {
                        // Просто используем обычную текстуру блока
                        if let Some(texture_id) = engine.get_ui_texture_id(texture_path) {
                            engine.add_ui_image(block_display.pos, block_display.size, texture_id);
                        } else {
                            // Загружаем текстуру если её нет
                            if let Some(texture_id) = engine.load_ui_texture(texture_path) {
                                engine.add_ui_image(block_display.pos, block_display.size, texture_id);
                            } else {
                                engine.add_ui_rect(block_display.pos, block_display.size, GRAY);
                            }
                        }
                    } else {
                        engine.add_ui_rect(block_display.pos, block_display.size, GRAY);
                    }
                } else {
                    engine.add_ui_rect(block_display.pos, block_display.size, RED);
                }
            }
        }
    }
    
    pub fn handle_click(&self, pos: Vec2) -> Option<String> {
        println!("[DEBUG] UI click at {:?}", pos);
        println!("[DEBUG] Checking {} buttons", self.buttons.len());
        
        for (id, button) in &self.buttons {
            if button.clickable && button.panel.visible {
                let panel = &button.panel;
                println!("[DEBUG] Button '{}' at {:?} size {:?}", id, panel.pos, panel.size);
                
                if pos.x >= panel.pos.x && pos.x <= panel.pos.x + panel.size.x &&
                   pos.y >= panel.pos.y && pos.y <= panel.pos.y + panel.size.y {
                    println!("[DEBUG] Button '{}' clicked!", id);
                    return Some(id.clone());
                }
            }
        }
        println!("[DEBUG] No button clicked");
        None
    }
}