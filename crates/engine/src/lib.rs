pub mod render;
pub mod input;
pub mod model;
pub mod cursor;
pub mod logger;
pub mod ui;
pub mod app;
pub mod app_runner;
pub mod runner;

pub use render::renderer::Renderer;
pub use render::ui::UIRenderer;
pub use render::image::ImageRenderer;
pub use input::InputEvent;
pub use model::vertex::Vertex;
pub use cursor::CursorManager;
pub use app::GameApp;
pub use runner::run_app;

use crate::logger::Logger;

use std::sync::Arc;
use winit::window::Window;
use glam::{Vec2, Vec4};

pub struct Engine<'window> {
    pub renderer: Renderer<'window>,
    pub image_renderer: ImageRenderer,
    pub cursor_manager: CursorManager,
    window: Arc<Window>,
    ui_textures: std::collections::HashMap<String, u32>,
}

impl<'window> Engine<'window> {
   
    pub fn new(window: Arc<Window>) -> Self {
        Logger::info("Initializing engine with textures");
        let renderer = Renderer::new(window.clone());
        let image_renderer = ImageRenderer::new(renderer.get_device(), renderer.get_surface_format());
        let cursor_manager = CursorManager::new();
        
        Self { renderer, image_renderer, cursor_manager, window, ui_textures: std::collections::HashMap::new() }
    }

    pub fn clear_meshes(&mut self) {
        self.renderer.clear_meshes();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    pub fn update_camera(&mut self, position: glam::Vec3, target: glam::Vec3, up: glam::Vec3) {
        self.renderer.set_camera_position(position);
        self.renderer.set_camera_target(target);
        self.renderer.set_camera_up(up);
    }
    
    pub fn lock_cursor(&mut self) {
        self.cursor_manager.lock_cursor(&self.window);
    }
    
    pub fn unlock_cursor(&mut self) {
        self.cursor_manager.unlock_cursor(&self.window);
    }
    
    pub fn toggle_cursor(&mut self) {
        self.cursor_manager.toggle_cursor(&self.window);
    }
    
    pub fn is_cursor_locked(&self) -> bool {
        self.cursor_manager.is_locked()
    }
    
    // pub fn clear_ui(&mut self) {
    //     self.ui_renderer.clear();
    // }
    
    pub fn load_ui_texture(&mut self, path: &str) -> Option<u32> {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let dimensions = rgba.dimensions();
            
            // Найдем свободный ID для текстуры (не трогаем блочные текстуры)
            let texture_id = self.ui_textures.len() as u32;
            
            // Загружаем текстуру в ImageRenderer
            self.image_renderer.load_texture(self.renderer.get_device(), self.renderer.get_queue(), texture_id, &rgba, dimensions);
            
            self.ui_textures.insert(path.to_string(), texture_id);
            Some(texture_id)
        } else {
            None
        }
    }
    
    pub fn get_ui_texture_id(&self, path: &str) -> Option<u32> {
        self.ui_textures.get(path).copied()
    }
}