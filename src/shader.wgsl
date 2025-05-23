
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct ModelUniform {
    model: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> model: ModelUniform;

// ---> Input Vertex Structure:
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

// ---> Output from fragment shader:
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}



@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // ---> Apply model transformation:
    let world_position = model.model * vec4<f32>(vertex.position, 1.0);

    // ---> Transform to clip-space:
    out.clip_position = camera.view_proj * world_position;
    out.frag_pos = world_position.xyz;

    // ---> Normal transformation (simplyfied... inverse transpose???)
    out.normal = (model.model * vec4<f32>(vertex.normal, 0.0)).xyz;
    out.tex_coords = vertex.tex_coords;

    return out;
}

@fragment // Simplified...
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ---> Simple illumination:
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(in.normal)

    // ---> Diffuse illumination:
    let diff = max(dot(normal, light_dir), 0.0);
    let ambient = 0.3;
    let light = ambient + diff * 0.7;

    // ---> Simple return (TODO: Textures...)
    return vec4<f32>(light, light, light, 1.0);
}