struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_uv: vec2f,
};

struct VertexOutput {
   @builtin(position) pos: vec4<f32>,
   @location(0) tex_uv: vec2f,
}

struct FragmentInput {
   @builtin(position) pos: vec4<f32>,
   @location(0) tex_uv: vec2f,
}

fn rotation_matrix_x(angle: f32) -> mat3x3<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return mat3x3<f32>(
        1.0, 0.0, 0.0,
        0.0,   c,  -s,
        0.0,   s,   c
    );
}

fn rotation_matrix_y(angle: f32) -> mat3x3<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return mat3x3<f32>(
         c, 0.0,   s,
        0.0, 1.0, 0.0,
         -s, 0.0,   c
    );
}

fn rotation_matrix_z(angle: f32) -> mat3x3<f32> {
    let s = sin(angle);
    let c = cos(angle);
    return mat3x3<f32>(
          c,  -s, 0.0,
          s,   c, 0.0,
        0.0, 0.0, 1.0
    );
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Apply model-view-projection transformation
    output.pos = uniforms.view_proj * uniforms.model * vec4<f32>(vertex.position, 1.0);
    output.tex_uv = vertex.tex_uv;
    
    return output;
}
@group(0) @binding(0)
var the_texture: texture_2d<f32>;
@group(0) @binding(1)
var the_sampler: sampler;
@group(1) @binding(0) 
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(fragment_in: FragmentInput) -> @location(0) vec4<f32> {
    //return vec4<f32>(uniforms.rotation, 0.0, 0.0, 1.0);
    return textureSample(the_texture, the_sampler, fragment_in.tex_uv);
}