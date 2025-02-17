use super::{mesh::Mesh, vertex::Vertex};

pub struct Cube {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
impl Cube {
    pub fn new() -> Self {
        let vertices = vec![
            Vertex::new([-0.5, -0.5,  0.5], [0.0, 0.0]),  // 0: front-bottom-left
            Vertex::new([ 0.5, -0.5,  0.5], [1.0, 0.0]),  // 1: front-bottom-right
            Vertex::new([ 0.5,  0.5,  0.5], [1.0, 1.0]),  // 2: front-top-right
            Vertex::new([-0.5,  0.5,  0.5], [0.0, 1.0]),  // 3: front-top-left
            
            // Back face vertices (z = -0.5)
            Vertex::new([-0.5, -0.5, -0.5], [1.0, 0.0]),  // 4: back-bottom-left
            Vertex::new([-0.5,  0.5, -0.5], [1.0, 1.0]),  // 5: back-top-left
            Vertex::new([ 0.5,  0.5, -0.5], [0.0, 1.0]),  // 6: back-top-right
            Vertex::new([ 0.5, -0.5, -0.5], [0.0, 0.0]),  // 7: back-bottom-right
        ];

        let indices = vec![
            // Front face (z = 0.5)
            0, 2, 1, // first triangle
            2, 3, 0, // second triangle
            // Right face (x = 0.5)
            //1, 7, 6, // first triangle
            //6, 2, 1, // second triangle
            // // Back face (z = -0.5)
            // 7, 4, 5, // first triangle
            // 5, 6, 7, // second triangle
            // // Left face (x = -0.5)
            // 4, 0, 3, // first triangle
            // 3, 5, 4, // second triangle
            // // Top face (y = 0.5)
            // 3, 2, 6, // first triangle
            // 6, 5, 3, // second triangle
            // // Bottom face (y = -0.5)
            // 4, 7, 1, // first triangle
            // 1, 0, 4, // second triangle
        ];
        Self { vertices, indices }
    }
}

impl Mesh for Cube {
    fn vertices(&self) -> Vec<Vertex> {
        self.vertices.clone()
    }

    fn indices(&self) -> Vec<u32> {
        self.indices.clone()
    }
}
