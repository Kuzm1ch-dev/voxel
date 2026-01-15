use mlua::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crate::modding::lua_block::LuaBlock;

pub struct ModInfo {
    pub name: String,
    pub version: String,
    pub author: String,
}

pub struct ModLoader {
    lua: Lua,
    pub blocks: HashMap<String, (LuaBlock, String)>,
}

impl ModLoader {
    pub fn new() -> LuaResult<Self> {
        Ok(Self {
            lua: Lua::new(),
            blocks: HashMap::new(),
        })
    }
    
    pub fn load_mods(&mut self, mods_dir: &str) -> LuaResult<()> {
        let mods_path = Path::new(mods_dir);
        
        if !mods_path.exists() {
            println!("Mods directory not found: {}", mods_dir);
            return Ok(());
        }
        
        for entry in fs::read_dir(mods_path).map_err(|e| LuaError::RuntimeError(e.to_string()))? {
            let entry = entry.map_err(|e| LuaError::RuntimeError(e.to_string()))?;
            let path = entry.path();
            
            if path.is_dir() {
                let init_lua = path.join("init.lua");
                if init_lua.exists() {
                    let mod_name = path.file_name().unwrap().to_string_lossy().to_string();
                    println!("Loading mod: {}", mod_name);
                    self.load_mod(init_lua.to_str().unwrap(), &path, &mod_name)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn load_mod(&mut self, lua_path: &str, mod_dir: &Path, mod_name: &str) -> LuaResult<()> {
        let blocks = Arc::new(Mutex::new(Vec::new()));
        let blocks_clone = Arc::clone(&blocks);
        let mod_dir_clone = mod_dir.to_path_buf();
        let mod_info = Arc::new(Mutex::new(None));
        let mod_info_clone = Arc::clone(&mod_info);
        
        let globals = self.lua.globals();
        let api_modloader = self.lua.create_table()?;
        
        let init_mod = self.lua.create_function(move |_, info_table: LuaTable| {
            let name: String = info_table.get("name")?;
            let version: String = info_table.get("version")?;
            let author: String = info_table.get("author")?;
            
            *mod_info_clone.lock().unwrap() = Some(ModInfo { name, version, author });
            
            Ok(())
        })?;
        
        let register_block = self.lua.create_function(move |_, block_table: LuaTable| {
            let id: String = block_table.get("id")?;
            let name: String = block_table.get("name")?;
            let texture: String = block_table.get("texture")?;
            let solid: bool = block_table.get("solid").unwrap_or(true);
            let transparent: bool = block_table.get("transparent").unwrap_or(false);
            
            let texture_path = if texture.starts_with("assets/") {
                mod_dir_clone.join(&texture).to_string_lossy().to_string()
            } else {
                texture
            };
            
            let lua_block = LuaBlock {
                id: id.clone(),
                name,
                texture_path,
                solid,
                transparent,
            };
            
            blocks_clone.lock().unwrap().push(lua_block);
            
            Ok(())
        })?;
        
        api_modloader.set("init", init_mod)?;
        api_modloader.set("register_block", register_block)?;
        globals.set("ModLoader", api_modloader)?;
        
        let code = fs::read_to_string(lua_path)
            .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
        
        self.lua.load(&code).exec()?;
        
        let info = mod_info.lock().unwrap();
        if let Some(mod_info) = info.as_ref() {
            println!("  Mod: {} v{} by {}", mod_info.name, mod_info.version, mod_info.author);
        }
        
        for block in blocks.lock().unwrap().iter() {
            println!("  Registered block: {} from mod {}", block.id, mod_name);
            self.blocks.insert(block.id.clone(), (block.clone(), mod_name.to_string()));
        }
        
        Ok(())
    }
}
