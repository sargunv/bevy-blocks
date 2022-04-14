#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] layer: i32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] layer: i32;
};

[[group(2), binding(0)]]
var<uniform> mesh: Mesh;

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.world_position = mesh.model * vec4<f32>(vertex.position, 1.0);
    out.world_normal = mat3x3<f32>(
        mesh.inverse_transpose_model[0].xyz,
        mesh.inverse_transpose_model[1].xyz,
        mesh.inverse_transpose_model[2].xyz
    ) * vertex.normal;

    out.clip_position = view.view_proj * out.world_position;

    out.uv = vertex.uv;
    out.layer = vertex.layer;

    return out;
}

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] layer: i32;
};

[[group(1), binding(0)]]
var base_color_texture: texture_2d_array<f32>;

[[group(1), binding(1)]]
var base_color_sampler: sampler;

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    return textureSample(base_color_texture, base_color_sampler, in.uv, in.layer);
}