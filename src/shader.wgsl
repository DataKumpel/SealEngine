///// UNIFORM STRUCTURES ///////////////////////////////////////////////////////////////////////////
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position : vec3<f32>,    // Camera position for specular...
    _pad     : f32,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct ModelUniform {
    model        : mat4x4<f32>,
    normal_matrix: mat3x3<f32>,  // Inverse transpose for normals...
};
@group(1) @binding(0) var<uniform> model: ModelUniform;
///// UNIFORM STRUCTURES ///////////////////////////////////////////////////////////////////////////

///// MATERIAL TEXTURES ////////////////////////////////////////////////////////////////////////////
@group(2) @binding(0) var diffuse_texture           : texture_2d<f32>;
@group(2) @binding(1) var diffuse_sampler           : sampler;
@group(2) @binding(2) var normal_texture            : texture_2d<f32>;
@group(2) @binding(3) var normal_sampler            : sampler;
@group(2) @binding(4) var metallic_roughness_texture: texture_2d<f32>;
@group(2) @binding(5) var metallic_roughness_sampler: sampler;
///// MATERIAL TEXTURES ////////////////////////////////////////////////////////////////////////////

///// LIGHT STRUCTURE //////////////////////////////////////////////////////////////////////////////
struct Light {
    position : vec3<f32>,
    range    : f32,
    color    : vec3<f32>, // RGB-color
    intensity: f32,
};
@group(3) @binding(0) var<uniform> light: Light;
///// LIGHT STRUCTURE //////////////////////////////////////////////////////////////////////////////

///// INPUT / OUTPUT STRUCTURES ////////////////////////////////////////////////////////////////////
// ---> Input Vertex Structure:
struct VertexInput {
    @location(0) position  : vec3<f32>,
    @location(1) normal    : vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) tangent   : vec3<f32>,
    @location(4) bitangent : vec3<f32>,
};

// ---> Output from fragment shader:
struct VertexOutput {
    @builtin(position)                             clip_position: vec4<f32>,
    @location(0) @interpolate(perspective, center) frag_pos     : vec3<f32>,
    @location(1) @interpolate(perspective, center) tex_coords   : vec2<f32>,
    @location(2) @interpolate(perspective, center) tangent      : vec3<f32>,
    @location(3) @interpolate(perspective, center) bitangent    : vec3<f32>,
    @location(4) @interpolate(perspective, center) normal       : vec3<f32>,
}
///// INPUT / OUTPUT STRUCTURES ////////////////////////////////////////////////////////////////////

///// VERTEX SHADER ////////////////////////////////////////////////////////////////////////////////
@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // ---> World space transformation:
    let world_position = model.model * vec4<f32>(vertex.position, 1.0);
    out.clip_position  = camera.view_proj * world_position;
    out.frag_pos       = world_position.xyz;
    out.tex_coords     = vertex.tex_coords;

    // ---> Construction of TBN Matrix:
    out.tangent   = normalize(model.normal_matrix * vertex.tangent);
    out.bitangent = normalize(model.normal_matrix * vertex.bitangent);
    out.normal    = normalize(model.normal_matrix * vertex.normal);

    return out;
}
///// VERTEX SHADER ////////////////////////////////////////////////////////////////////////////////

///// FRAGMENT SHADER //////////////////////////////////////////////////////////////////////////////
@fragment // Simplified...
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ---> Material properties:
    let diffuse_color      = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    let metallic_roughness = textureSample(metallic_roughness_texture, 
                                           metallic_roughness_sampler,
                                           in.tex_coords);

    // ---> Normal mapping:
    let tangent_normal = textureSample(normal_texture, normal_sampler, in.tex_coords).rgb * 2.0 - 1.0;
    let tbn_matrix     = mat3x3<f32>(in.tangent, in.bitangent, in.normal);
    let world_normal   = normalize(tbn_matrix * tangent_normal);

    // ---> Light calculation:
    let light_dir   = normalize( light.position - in.frag_pos);
    let view_dir    = normalize(camera.position - in.frag_pos);
    let halfway_dir = normalize(light_dir + view_dir);

    // ---> Diffuse component:
    let diff = max(dot(world_normal, light_dir), 0.0);

    // ---> Specular component (Blinn-Phong):
    let spec = pow(max(dot(world_normal, halfway_dir), 0.0), 32.0);

    // ---> Attenuation:
    let distance    = length(light.position - in.frag_pos);
    let attenuation = 1.0 / (distance * distance);

    // ---> Combine components:
    let ambient     = 0.1 * diffuse_color.rgb;
    let diffuse     = diff * light.color * light.intensity * attenuation;
    let specular    = spec * light.color * light.intensity * attenuation;
    let final_color = (ambient + diffuse + specular) * diffuse_color.rgb;
    
    return vec4<f32>(final_color, diffuse_color.a);
}
///// FRAGMENT SHADER //////////////////////////////////////////////////////////////////////////////