use super::vertex::Vertex;

pub trait Mesh {
    fn vertices(&self) -> Vec<Vertex>;
    fn indices(&self) -> Vec<u32>;
}

