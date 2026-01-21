struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    out.uv = input.uv;
    return out;
}

@group(0) @binding(0)
var t_texture: texture_2d<f32>;
@group(0) @binding(1)
var s_texture: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_texture, s_texture, input.uv);
}