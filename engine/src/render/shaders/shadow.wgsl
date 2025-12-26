struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    params: vec4<f32>,
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> light_uniform: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = light_uniform.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.clip_position.z, 0.0, 0.0, 1.0);
}
