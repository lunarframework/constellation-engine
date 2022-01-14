var<private> vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
);

struct VertexInput {
    // xyz: position, w: radius
    [[location(0)]]
    pos: vec4<f32>;
    [[location(1)]]
    color: vec4<f32>;
    [[location(2)]]
    shift: vec4<f32>;

    [[builtin(vertex_index)]] vertex_index: u32;
};

struct VertexData {
    [[location(0)]]
    color: vec4<f32>;
    [[location(1)]]
    shift: vec4<f32>;
    [[location(2)]]
    coords: vec2<f32>;
    [[builtin(position)]] pos: vec4<f32>;
};

struct Enviornment {
    proj: mat4x4<f32>;
    view: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> env: Enviornment;

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexData {
    let vertex = vertices[in.vertex_index];
    let viewspace = env.view * vec4<f32>(in.pos.xyz, 1.0) + vec4<f32>(vertex * in.pos.w, 0.0, 0.0);

    var out: VertexData;
    out.color = in.color;
    out.shift = in.pos;
    out.coords = vertex;
    out.pos = env.proj * viewspace;
    return out;
}

struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexData) -> FragmentOutput { 
    let product = in.coords * in.coords;
    let r2 = product.x + product.y;

    if (r2 > 1.0) {
        discard;
    }

    var out: FragmentOutput;
    out.color = vec4<f32>(in.color.xyz, 1.0);
    return out;
}