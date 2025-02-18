
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightViewProj {
    view_proj: [[f32; 4]; 4],
}

impl LightViewProj {
    pub fn new(view_proj: [[f32; 4]; 4]) -> Self {
        Self { view_proj }
    }
}

#[repr(C, align(16))] // Explicitly align to 16 bytes
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    direction: [f32; 4],    // 16 bytes (using vec4 alignment)
    color: [f32; 4],       // 16 bytes (using vec4 alignment)
    params: [f32; 4],      // 16 bytes for intensity, ambient_strength, and padding
}

impl LightUniform {
    pub fn new(direction: [f32; 3], color: [f32; 3], intensity: f32, ambient_strength: f32) -> Self {
        Self {
            direction: [direction[0], direction[1], direction[2], 0.0],
            color: [color[0], color[1], color[2], 0.0],
            params: [intensity, ambient_strength, 0.0, 0.0],
        }
    }
}