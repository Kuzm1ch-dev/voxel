use voxel_engine::Engine;

pub struct TextureLoader;

impl TextureLoader {
    pub fn load_block_textures(engine: &mut Engine) {
        let texture_paths = [
            "game/assets/textures/blocks/stone.png",
            "game/assets/textures/blocks/dirt.png", 
            "game/assets/textures/blocks/grass.png",
            "game/assets/textures/blocks/block.png",
        ];
        
        // Try to load actual textures
        for (i, path) in texture_paths.iter().enumerate() {
            if let Ok(img) = image::open(path) {
                let rgba = img.to_rgba8();
                let dimensions = rgba.dimensions();
                
                // Update texture in engine
                engine.renderer.update_texture_layer(i as u32, &rgba, dimensions);
            }
        }
    }
}