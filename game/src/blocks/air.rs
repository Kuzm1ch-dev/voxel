use crate::common::block::Block;

#[derive(Clone)]
pub struct AirBlock;

impl Block for AirBlock {
    fn get_id(&self) -> &'static str { "air" }
    fn get_name(&self) -> &'static str { "air" }
    fn get_texture_path(&self) -> &'static str { "" }
    fn is_solid(&self) -> bool { false }
    fn is_transparent(&self) -> bool { true }
}
