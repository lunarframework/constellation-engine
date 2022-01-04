var<private> vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
);

struct VertexInput {
    [[builtin(vertex_index)]] vertex_index: u32;
};

struct VertexData {
    [[builtin(position)]] pos: vec4<f32>;
};


[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexData {
    var out: VertexData;
    out.pos = vec4<f32>(vertices[in.vertex_index], 0.0, 1.0);
    return out;
}

struct FragmentOutput {
    [[builtin(frag_depth)]] depth: f32;
    [[location(0)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexData) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = in.pos;
    out.depth = 1.0;
    return out;
}