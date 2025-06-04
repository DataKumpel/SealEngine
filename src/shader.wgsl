///// UNIFORM STRUCTURES ///////////////////////////////////////////////////////////////////////////
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct ModelUniform {
    model: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> model: ModelUniform;
///// UNIFORM STRUCTURES ///////////////////////////////////////////////////////////////////////////

///// MATERIAL TEXTURES ////////////////////////////////////////////////////////////////////////////
@group(2) @binding(0) var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1) var diffuse_sampler: sampler;
@group(2) @binding(2) var normal_texture: texture_2d<f32>;
@group(2) @binding(3) var normal_sampler: sampler;
///// MATERIAL TEXTURES ////////////////////////////////////////////////////////////////////////////

///// INPUT / OUTPUT STRUCTURES ////////////////////////////////////////////////////////////////////
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
    @location(3) world_normal: vec3<f32>,
}
///// INPUT / OUTPUT STRUCTURES ////////////////////////////////////////////////////////////////////

///// VERTEX SHADER ////////////////////////////////////////////////////////////////////////////////
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
    out.world_normal = out.normal;  // TODO: later...
    out.tex_coords = vertex.tex_coords;

    return out;
}
///// VERTEX SHADER ////////////////////////////////////////////////////////////////////////////////

///// FRAGMENT SHADER //////////////////////////////////////////////////////////////////////////////
@fragment // Simplified...
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ---> Sample diffuse texture:
    let diffuse_color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);

    // ---> Sample normal-map (if exists):
    let normal_map = textureSample(normal_texture, normal_sampler, in.tex_coords);

    // ---> Convert normal-map from [0,1] to [-1,1]:
    let tangent_normal = normal_map.rgb * 2.0 - 1.0;

    // ---> Normal for illumination (TODO: TBN-Matrix):
    let normal = normalize(in.normal + tangent_normal.xyz * 0.5);

    // ---> Diffuse illumination:
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));  // Light from top right...
    let diff = max(dot(normal, light_dir), 0.0);
    let ambient = 0.3;
    let light_intensity = ambient + diff * 0.7;

    // ---> Combine texture-color with light intensity:
    let final_color = diffuse_color.rgb * light_intensity;
    
    return vec4<f32>(final_color, diffuse_color.a);
}
///// FRAGMENT SHADER //////////////////////////////////////////////////////////////////////////////