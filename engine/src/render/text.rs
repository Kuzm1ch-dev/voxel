use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};
use glam::{Vec2, Vec4};

pub struct TextRenderer {
    glyph_brush: GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
}

impl TextRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, font_path: &str) -> Self {
        let font_data = std::fs::read(font_path)
            .unwrap_or_else(|_| panic!("Не удалось загрузить шрифт: {}", font_path));
        
        let font = ab_glyph::FontArc::try_from_slice(&font_data)
            .unwrap_or_else(|_| panic!("Неверный формат шрифта: {}", font_path));

        let glyph_brush = GlyphBrushBuilder::using_font(font).build(device, surface_format);
        let staging_belt = wgpu::util::StagingBelt::new(1024);

        Self {
            glyph_brush,
            staging_belt,
        }
    }

    pub fn queue_text(&mut self, text: &str, pos: Vec2, size: f32, color: Vec4, screen_size: (u32, u32)) {
        let section = Section::default()
            .add_text(Text::new(text).with_color([color.x, color.y, color.z, color.w]).with_scale(size))
            .with_screen_position((pos.x * screen_size.0 as f32, pos.y * screen_size.1 as f32));

        self.glyph_brush.queue(section);
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        screen_size: (u32, u32),
    ) -> Result<(), String> {
        self.glyph_brush
            .draw_queued(
                device,
                &mut self.staging_belt,
                encoder,
                target,
                screen_size.0,
                screen_size.1,
            )
            .map_err(|e| format!("Text render error: {:?}", e))?;

        self.staging_belt.finish();
        Ok(())
    }

    pub fn recall_staging_belt(&mut self) {
        self.staging_belt.recall();
    }
}