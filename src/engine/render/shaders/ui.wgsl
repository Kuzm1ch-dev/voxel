struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Convert normalized screen coordinates (0-1) to NDC (-1 to 1)
    // Y флип для правильной ориентации
    out.clip_position = vec4<f32>(
        model.position.x * 2.0 - 1.0,
        1.0 - model.position.y * 2.0,
        0.0, 
        1.0
    );
    out.uv = model.uv;
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}