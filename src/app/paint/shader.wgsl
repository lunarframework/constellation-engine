[[block]]
struct Uniforms {
    uscale: vec2<f32>;
    utranslate: vec2<f32>;
};

struct VertexInput {
    [[location(0)]] a_Pos: vec2<f32>;
    [[location(1)]] a_UV: vec2<f32>;
    [[location(2)]] a_Color: vec4<f32>;
};

struct VertexOutput {
    [[location(0)]] v_UV: vec2<f32>;
    [[location(1)]] v_Color: vec4<f32>;
    [[builtin(position)]] v_Position: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.v_UV = in.a_UV;
    out.v_Color = in.a_Color;
    out.v_Position = vec4<f32>(in.a_Pos.xy * uniforms.uscale + uniforms.utranslate, 0.0, 1.0);
    return out;
}

struct FragmentOutput {
    [[location(0)]] o_Target: vec4<f32>;
};

[[group(1), binding(0)]]
var u_Texture: texture_2d<f32>;
[[group(1), binding(1)]]
var u_Sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let color = in.v_Color;

    return FragmentOutput(color * textureSample(u_Texture, u_Sampler, in.v_UV));
}