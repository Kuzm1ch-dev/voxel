pub mod render;
pub mod input;
pub mod model;
pub mod cursor;

pub use render::renderer::Renderer;
pub use render::ui::UIRenderer;
pub use render::bitmap_font::BitmapFont;
pub use render::image::ImageRenderer;
pub use input::InputEvent;
pub use model::vertex::Vertex;
pub use cursor::CursorManager;

use std::sync::Arc;
use winit::window::Window;
use glam::{Vec2, Vec4};

pub struct Engine<'window> {
    pub renderer: Renderer<'window>,
    pub ui_renderer: UIRenderer,
    pub image_renderer: ImageRenderer,
    pub cursor_manager: CursorManager,
    window: Arc<Window>,
    pub grass_texture: Option<(wgpu::Texture, wgpu::TextureView)>,
}

impl<'window> Engine<'window> {
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new_with_textures(window.clone(), &[]);
        let ui_renderer = UIRenderer::new(renderer.get_device(), renderer.get_surface_format());
        let image_renderer = ImageRenderer::new(renderer.get_device(), renderer.get_surface_format());
        let cursor_manager = CursorManager::new();
        Self { renderer, ui_renderer, image_renderer, cursor_manager, window, grass_texture: None }
    }
    
    pub fn new_with_textures(window: Arc<Window>, texture_paths: &[String]) -> Self {
        let renderer = Renderer::new_with_textures(window.clone(), texture_paths);
        let ui_renderer = UIRenderer::new(renderer.get_device(), renderer.get_surface_format());
        let image_renderer = ImageRenderer::new(renderer.get_device(), renderer.get_surface_format());
        let cursor_manager = CursorManager::new();
        
        // Load grass texture
        let grass_texture = if let Ok(img) = image::open("game/assets/textures/block/grass.png") {
            let rgba = img.to_rgba8();
            let dimensions = rgba.dimensions();
            
            let texture = renderer.get_device().create_texture(&wgpu::TextureDescriptor {
                label: Some("Grass Texture"),
                size: wgpu::Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            
            renderer.get_queue().write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                wgpu::Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 },
            );
            
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            Some((texture, view))
        } else {
            None
        };
        
        Self { renderer, ui_renderer, image_renderer, cursor_manager, window, grass_texture }
    }

    pub fn clear_meshes(&mut self) {
        self.renderer.clear_meshes();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render_with_ui(&mut self.ui_renderer)
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
    
    pub fn clear_ui(&mut self) {
        self.ui_renderer.clear();
    }
    
    pub fn add_ui_rect(&mut self, pos: Vec2, size: Vec2, color: Vec4) {
        self.ui_renderer.add_rect(pos, size, color);
    }
    

}