struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
};

[[block]] struct Locals {
    proj_view_model: mat4x4<f32>;
};
[[group(0), binding(0)]] var<uniform> r_locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] a_pos: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = r_locals.proj_view_model * vec4<f32>(a_pos, 1.0);

    return out;
}

// Fragment shader bindings

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}