
pub trait Block {
    fn get_id(&self) -> &'static str;
    fn get_name(&self) -> &'static str;
    fn get_texture_path(&self) -> &'static str;
    fn is_solid(&self) -> bool;
    fn is_transparent(&self) -> bool;
}