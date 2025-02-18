struct CameraUniform {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    params: vec4<f32>,
}

struct LightViewProj {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> light: LightUniform;
@group(2) @binding(0) var shadow_texture: texture_depth_2d;
@group(2) @binding(1) var shadow_sampler: sampler_comparison;
@group(3) @binding(0) var<uniform> light_view_proj: LightViewProj;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) shadow_coords: vec3<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position
    let model_matrix = camera.model;
    let world_position = (model_matrix * vec4<f32>(model.position, 1.0)).xyz;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    out.world_position = world_position;
    
    // Transform normal
    out.world_normal = (model_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    out.uv = model.uv;
    
    // Calculate shadow coordinates
    let shadow_pos = light_view_proj.view_proj * vec4<f32>(world_position, 1.0);
    out.shadow_coords = vec3<f32>(
        shadow_pos.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5),
        shadow_pos.z
    );
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let light_dir = normalize(light.direction.xyz);
    
    // Calculate shadow factor
    var shadow: f32 = 0.0;
    let size = f32(1.0) / 1024.0; // shadow map size
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            let offset = vec2<f32>(f32(x) * size, f32(y) * size);
            shadow += textureSampleCompare(
                shadow_texture,
                shadow_sampler,
                in.shadow_coords.xy + offset,
                in.shadow_coords.z - 0.001 // bias to reduce shadow acne
            );
        }
    }
    shadow /= 4.0; // Average the samples
    
    // Calculate lighting
    let ambient = light.color.xyz * light.params.y; // ambient_strength is params.y
    let diff = max(dot(normal, -light_dir), 0.0);
    let diffuse = light.color.xyz * diff * light.params.x; // intensity is params.x
    
    // Base color (you can modify this or add texture sampling)
    let base_color = vec3<f32>(0.8, 0.8, 0.8);
    
    // Combine lighting with shadows
    let lighting = ambient + diffuse * shadow;
    let final_color = base_color * lighting;
    
    return vec4<f32>(final_color, 1.0);
}
