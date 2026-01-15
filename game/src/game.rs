use voxel_engine::{Engine, GameApp, InputEvent};
use glam::Vec2;
use crate::game_state::{self, GameState};
use crate::systems::render_system::RenderSystem;
use crate::systems::input_system::InputSystem;
use crate::systems::ui_system::UISystem;
use crate::modding::mod_loader::ModLoader;

pub struct Game {
    game_state: Option<GameState>,
    ui_system: Option<UISystem>,
    render_system: Option<RenderSystem>,
    input_system: Option<InputSystem>,
    mod_loader: Option<ModLoader>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            game_state: None,
            ui_system: None,
            render_system: None,
            input_system: None,
            mod_loader: None,
        }
    }
}

impl GameApp for Game {
    fn ready(&mut self, engine: &mut Engine) {
        // Load mods first
        let mut mod_loader = ModLoader::new().expect("Failed to create mod loader");
        if let Err(e) = mod_loader.load_mods("mods") {
            eprintln!("Error loading mods: {}", e);
        }
        
        let mut game_state = GameState::new(engine);
        
        // Register Lua blocks
        for (id, (lua_block, mod_name)) in mod_loader.blocks.iter() {
            println!("Registering block '{}' from mod '{}'", id, mod_name);
            game_state.world.registry.register_lua_block(lua_block.clone(), engine);
        }
        
        // Load textures
        game_state.world.registry.load_textures(engine);
        
        engine.lock_cursor();
        
        self.game_state = Some(game_state);
        self.ui_system = Some(UISystem::new());
        self.render_system = Some(RenderSystem::new());
        self.input_system = Some(InputSystem::new());
        self.mod_loader = Some(mod_loader);
    }
    
    fn update(&mut self, engine: &mut Engine, delta_time: f32) {
        if let (Some(game_state), Some(ui_system)) = (self.game_state.as_mut(), self.ui_system.as_mut()) {
            game_state.update(engine, delta_time, ui_system.is_open);
            
            if ui_system.is_open {
                engine.unlock_cursor();
            } else {
                engine.lock_cursor();
            }
        }
    }
    
    fn input_event(&mut self, engine: &mut Engine, event: &InputEvent) {
        if let (Some(game_state), Some(ui_system), Some(input_system)) = 
            (self.game_state.as_mut(), self.ui_system.as_mut(), self.input_system.as_ref()) 
        {
            input_system.handle_input(engine, event, game_state, ui_system);
        }
    }
    
    fn render(&mut self, engine: &mut Engine) {
        if let (Some(game_state), Some(ui_system), Some(render_system)) = 
            (self.game_state.as_mut(), self.ui_system.as_mut(), self.render_system.as_ref()) 
        {
            let _ = render_system.render(engine, game_state, ui_system);
        }
    }
    
    fn resize(&mut self, engine: &mut Engine, new_size: winit::dpi::PhysicalSize<u32>) {
        engine.resize(new_size);
    }
}