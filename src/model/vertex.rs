use wgpu::naga::proc::index;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    tex_uv: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], tex_uv: [f32; 2]) -> Self {
        Self { position, tex_uv }
    }
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

pub fn create_box() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::new([-0.5, -0.5,  0.5], [0.0, 0.0]),  // 0
        Vertex::new([ 0.5, -0.5,  0.5], [1.0, 0.0]),  // 1
        Vertex::new([ 0.5,  0.5,  0.5], [1.0, 1.0]),  // 2
        Vertex::new([-0.5,  0.5,  0.5], [0.0, 1.0]),  // 3
        Vertex::new([-0.5, -0.5, -0.5], [1.0, 0.0]),  // 4
        Vertex::new([-0.5,  0.5, -0.5], [1.0, 1.0]),  // 5
        Vertex::new([ 0.5,  0.5, -0.5], [0.0, 1.0]),  // 6
        Vertex::new([ 0.5, -0.5, -0.5], [0.0, 0.0]),  // 7
    ];

    let indices = vec![
        // Front
        0, 1, 2, 
        2, 3, 0,
        // Right
        1, 7, 6, 
        6, 2, 1,
        // Back
        7, 4, 5, 
        5, 6, 7,
        // Left
        4, 0, 3, 
        3, 5, 4,
        // Top
        3, 2, 6, 
        6, 5, 3,
        // Bottom
        4, 7, 1, 
        1, 0, 4,
    ];

    (vertices, indices)
}

pub fn create_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    }
}
