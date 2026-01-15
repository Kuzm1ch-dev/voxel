use crate::common::block::Block;

#[derive(Clone)]
pub struct LuaBlock {
    pub id: String,
    pub name: String,
    pub texture_path: String,
    pub solid: bool,
    pub transparent: bool,
}

impl Block for LuaBlock {
    fn get_id(&self) -> &'static str {
        Box::leak(self.id.clone().into_boxed_str())
    }
    
    fn get_name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }
    
    fn get_texture_path(&self) -> &'static str {
        Box::leak(self.texture_path.clone().into_boxed_str())
    }
    
    fn is_solid(&self) -> bool {
        self.solid
    }
    
    fn is_transparent(&self) -> bool {
        self.transparent
    }
}
