use crate::common::block::Block;

#[derive(Clone)]
pub struct GrassBlock;

impl Block for GrassBlock {
    fn get_id(&self) -> &'static str { "grass" }
    fn get_name(&self) -> &'static str { "grass" }
    fn get_texture_path(&self) -> &'static str { "assets/textures/block/grass.png" }
    fn is_solid(&self) -> bool { true }
    fn is_transparent(&self) -> bool { false }
}