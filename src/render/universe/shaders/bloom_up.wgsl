[[group(0), binding(0)]]
var<uniform> output: texture_storage_2d<rgba32float, write>;

// When upsampling this the texture downsampled texture.
[[group(0), binding(1)]]
var<uniform> downsampled: texture_2d<f32>;

// Texture with the previous mip of upsampled
[[group(0), binding(2)]]
var<uniform> upsampled: texture2d<f32>;

[[group(0), binding(3)]]
var<uniform> samp: sampler;

[[block]]
struct Uniforms {
    lod: f32;
};

[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;

fn upsample_tent9(t: texture_2d<f32>, s: sampler, lod: f32, uv: vec2<f32>, texel_size: vec2<f32>, radius: f32) -> vec3<f32> {
    let offset = texel_size.xyxy * vec4<f32>(1.0, 1.0, -1.0, 0.0) * radius;

    // Center
    var result = textureSampleLevel(t, s, uv, lod).rgb * 4.0;

    result += textureSampleLevel(t, s, uv - offset.xy, lod).rgb;
    result += textureSampleLevel(t, s, uv - offset.wy, lod).rgb * 2.0;
    result += textureSampleLevel(t, s, uv - offset.zy, lod).rgb;

    result += textureSampleLevel(t, s, uv + offset.zw, lod).rgb * 2.0;
    result += textureSampleLevel(t, s, uv + offset.xw, lod).rgb * 2.0;

    result += textureSampleLevel(t, s, uv + offset.zy, lod).rgb;
    result += textureSampleLevel(t, s, uv + offset.wy, lod).rgb * 2.0;
    result += textureSampleLevel(t, s, uv + offset.xy, lod).rgb;

    return result * (1.0 / 6.0);
}

let SAMPLE_SCALE: f32 = 1.0;

[[stage(compute), workgroup_size_x(4), workgroup_size_y(4)]]
fn main([[builtin(global_invocation_id)]] invocation: vec3<u32>) {
    let output_size = vec2<f32>(textureDimensions(output));

    var tex_coords = vec2<f32>(f32(invocation.x) / output_size.x, f32(invocation.y) / output_size.y);
    tex_coords += (1.0 / output_size) * 0.5;

    var color: vec4<f32>;

    let upsampled_size = vec2<f32>(textureDimensions(upsampled, i32(uniforms.lod + 1.0)));
    let upsampled_color = upsample_tent9(upsampled, samp, uniforms.lod + 1.0, tex_coords, 1.0 / upsampled_size, SAMPLE_SCALE);

    let existing = textureSampleLevel(downsampled, samp, tex_coords, uniforms.lod).rgb;
    color.rgb = existing + upsampled_texture;
    color.a = 1.0;

    textureStore(output, vec2<i32>(invocation), color);
}