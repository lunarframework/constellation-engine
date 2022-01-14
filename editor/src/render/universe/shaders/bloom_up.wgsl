[[group(0), binding(0)]]
var output: texture_storage_2d<rgba16float, write>;

// When upsampling this the texture downsampled texture.
[[group(0), binding(1)]]
var downsampled: texture_2d<f32>;

// Texture with the previous mip of upsampled
[[group(0), binding(2)]]
var upsampled: texture_2d<f32>;

[[group(0), binding(3)]]
var samp: sampler;

struct Uniforms {
    lod: f32;
};

[[group(0), binding(4)]]
var<uniform> uniforms: Uniforms;

fn upsample_tent9(t: texture_2d<f32>, s: sampler, lod: f32, uv: vec2<f32>, texel_size: vec2<f32>, radius: f32) -> vec3<f32> {
    let offset = texel_size.xyxy * vec4<f32>(1.0, 1.0, -1.0, 0.0) * radius;

    // Center
    var result = vec3<f32>(0.0, 0.0, 0.0);

    result = result + textureSampleLevel(t, s, uv - offset.xy, lod).rgb;
    result = result + textureSampleLevel(t, s, uv - offset.wy, lod).rgb * 2.0;
    result = result + textureSampleLevel(t, s, uv - offset.zy, lod).rgb;

    result = result + textureSampleLevel(t, s, uv + offset.zw, lod).rgb * 2.0;
    result = result + textureSampleLevel(t, s, uv, lod).rgb * 4.0;
    result = result + textureSampleLevel(t, s, uv + offset.xw, lod).rgb * 2.0;

    result = result + textureSampleLevel(t, s, uv + offset.zy, lod).rgb;
    result = result + textureSampleLevel(t, s, uv + offset.wy, lod).rgb * 2.0;
    result = result + textureSampleLevel(t, s, uv + offset.xy, lod).rgb;

    return result * (1.0 / 16.0);
}

// fn upsample_box4(t: texture_2d<f32>, s: sampler, lod: f32, uv: vec2<f32>, texel_size: vec2<f32>, radius: f32) -> vec3<f32> {
//     let offset = texel_size.xyxy * vec4<f32>(-1.0, -1.0, 1.0, 1.0) * radius;

//     // Center
//     var result = vec3<f32>(0.0, 0.0, 0.0);

//     result = result + textureSampleLevel(t, s, uv + offset.xy, lod).rgb;
//     result = result + textureSampleLevel(t, s, uv + offset.zy, lod).rgb;
//     result = result + textureSampleLevel(t, s, uv + offset.xw, lod).rgb;
//     result = result + textureSampleLevel(t, s, uv + offset.zw, lod).rgb;

//     return result * (1.0 / 4.0);
// }

let SAMPLE_SCALE: f32 = 1.0;

[[stage(compute), workgroup_size(4, 4, 1)]]
fn main([[builtin(global_invocation_id)]] invocation: vec3<u32>) {
    let output_size = vec2<f32>(textureDimensions(output));

    var coords = vec2<f32>(invocation.xy) / output_size;
    coords = coords + (1.0 / output_size) * 0.5;

    let upsampled_size = vec2<f32>(textureDimensions(upsampled, i32(uniforms.lod + 1.0)));
    let upsampled_color = upsample_tent9(upsampled, samp, uniforms.lod + 1.0, coords, 1.0 / upsampled_size, SAMPLE_SCALE);

    let existing = textureSampleLevel(downsampled, samp, coords, uniforms.lod).rgb;
    let color = vec4<f32>(existing + upsampled_color, upsampled_color.r);

    textureStore(output, vec2<i32>(invocation.xy), color);
}