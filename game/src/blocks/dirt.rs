use crate::common::block::Block;

#[derive(Clone)]
pub struct DirtBlock;

impl Block for DirtBlock {
    fn get_id(&self) -> &'static str { "dirt" }
    fn get_name(&self) -> &'static str { "dirt" }
    fn get_texture_path(&self) -> &'static str { "assets/textures/block/dirt.png" }
    fn is_solid(&self) -> bool { true }
    fn is_transparent(&self) -> bool { false }
}