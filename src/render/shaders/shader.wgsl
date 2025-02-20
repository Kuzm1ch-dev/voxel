struct CameraUniform {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    view_position: vec3<f32>
}

struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    params: vec4<f32>,
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> light: LightUniform;
@group(2) @binding(0) var shadow_texture: texture_depth_2d;
@group(2) @binding(1) var shadow_sampler: sampler_comparison;
@group(3) @binding(0) var texture_array: texture_2d_array<f32>;
@group(3) @binding(1) var texture_sampler: sampler;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) shadow_coords: vec3<f32>,
    @location(4) tex_index: u32,  // Pass texture index to fragment shader
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
    out.tex_index = model.tex_index;

    // Calculate shadow coordinates
    let shadow_pos = light.view_proj * vec4<f32>(world_position, 1.0);
    out.shadow_coords = vec3<f32>(
        shadow_pos.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5),
        shadow_pos.z
    );
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(camera.view_position.xyz - in.world_position);
    let normal = normalize(in.world_normal);
    let light_dir = normalize(light.direction.xyz);
        let base_color = textureSample(
        texture_array, 
        texture_sampler, 
        in.uv, 
        in.tex_index
    ).rgb;

    // Calculate shadow factor
    var shadow: f32 = 0.0;
    let size = f32(1.0) / 2048.0; // shadow map size
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
    shadow /= 9.0; // Average the samples
    
    // Calculate lighting
    let ambient_strength = light.params.y;
    let diffuse_intensity = light.params.x;
    let ambient = light.color.xyz * ambient_strength; // ambient_strength is params.y
    let diff = max(dot(normal, -light_dir), 0.0);
    let diffuse = light.color.xyz * diff * diffuse_intensity; // intensity is params.x

    // Base color (you can modify this or add texture sampling)
    // let base_color = vec3<f32>(0.8, 0.8, 0.8);
    //rim lighting
    var rim = 1.0 - max(dot(normal, view_dir), 0.0);
    rim = smoothstep(0.0, 1.0, rim); // Сглаживание эффекта
    let rim_light = vec3<f32>(0.05) * rim; // Интенсивность краевого освещения

    // Combine lighting with shadows
    let lighting = ambient + diffuse * shadow;
    let final_color = base_color * lighting + rim_light;
    return vec4<f32>(final_color, 1.0);
}
