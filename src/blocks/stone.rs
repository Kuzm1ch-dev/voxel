use crate::common::block::Block;

#[derive(Clone)]
pub struct StoneBlock;

impl Block for StoneBlock {
    fn get_id(&self) -> &'static str { "stone" }
    fn get_name(&self) -> &'static str { "stone" }
    fn get_texture_path(&self) -> &'static str { "assets/textures/block/stone.png" }
    fn is_solid(&self) -> bool { true }
    fn is_transparent(&self) -> bool { false }
}
