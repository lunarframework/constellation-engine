let EPISLON: f32 = 1.0e-4;
let MODE_PREFILTER: u32 = 0;
let MODE_DOWNSAMPLE: u32 = 1;
let MODE_UPSAMPLE_FIRST: u32 = 2;
let MODE_UPSAMPLE: u32 = 3;

[[group(0), binding(0)]]
var<uniform> output: texture_storage_2d<rgba32float, write>;

// Input texture 
// When prefiltering this is the raw hdr texture of the scene
// When downsampling this is the texture to downsample from
// When upsampling this the texture downsampled texture.
[[group(0), binding(1)]]
var<uniform> input: texture_2d<f32>;

// Texture with the previous mip of upsampled
[[group(0), binding(2)]]
var<uniform> upsampled: texture2d<f32>;

[[group(0), binding(3)]]
var<uniform> samp: sampler;

[[block]]
struct Uniforms {
    threshold: f32;
    knee: f32;
    lod: f32;
    mode: u32;
};

[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;

fn downsample_box13(t: texture_2d<f32>, s: sampler, lod: f323, uv: vec2<f32>, texel_size: vec2<f32>) -> vec3<f32> {
    // Center
    let a = textureSampleLevel(t, s, uv, lod).rgb;

    let texel_size = texel_size * 0.5; // Sample from center of texels
    
    // Inner box
    let b = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(-1.0, -1.0), lod).rgb;
    let c = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(-1.0, 1.0), lod).rgb;
    let d = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(1.0, 1.0), lod).rgb;
    let e = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(1.0, -1.0), lod).rgb;

    // Outer box
    let f = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(-2.0, -2.0), lod).rgb;
    let g = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(-2.0, 0.0), lod).rgb;
    let h = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(0.0, 2.0), lod).rgb;
    let i = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(2.0, 2.0), lod).rgb;
    let j = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(2.0, 2.0), lod).rgb;
    let k = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(2.0, 0.0), lod).rgb;
    let l = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(-2.0, -2.0), lod).rgb;
    let m = textureSampleLevel(t, s, uv + texelSize * vec2<f32>(0.0, -2.0), lod).rgb;

    // Weights
    var result = vec3<f32>(0.0);
    // Inner box
    result += (b + c + d + e) * 0.5f;
    // Bottom-left box
    result += (f + g + a + Mm) * 0.125f;
    // Top-left box
    result += (g + h + i + a) * 0.125f;
    // Top-right box
    result += (a + i + j + k) * 0.125f;
    // Bottom-right box
    result += (m + a + k + l) * 0.125f;

    // 4 samples each
    result *= 0.25f;

    return result;
}

// Quadratic color thresholding
// curve = (threshold - knee, knee * 2, 0.25 / knee)
fn quadratic_threshold(color: vec4<f32>, threshold: f32, curve: vec3<f32>) -> vec4<f32> {
    // Maximum pixel brightness
    let brightness = max(max(color.r, color.g), color.b);
    // Quadratic curve
    let rq = clamp(brightness - curve.x, 0.0, curve.y);
    let rq2 = rq * rq * curve.z;
    return color * max(rq2, brightness - threshold) / max(brightness, EPISLON);
}

fn prefilter(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let clamp_value = 20.0;
    let clamped_color = min(vec4<f32>(clamp_value), color);
    
    return quadratic_threshold(clamped_color, uniforms.threshold, vec3<f32>(uniforms.threshold - uniforms.knee, 2.0 * uniforms.knee, 0.25 / uniforms.knee));
}

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

    let input_size = vec2<f32>(textureDimensions(input, i32(uniforms.lod)));
    var color = vec4<f32>(1.0, 0.0, 1.0, 1.0);

    if (uniforms.mode == MODE_PREFILTER) {
        color.rgb = downsample_box13(input, samp, 0.0, tex_coords, 1.0 / input_size);
        color = prefilter(color, tex_coords);
        color.a = 1.0;
    } else if (uniforms.mode == MODE_DOWNSAMPLE) {
        // Downsample
        color.rgb = downsample_box13(input, samp, uniforms.lod, tex_coords, 1.0 / input_size);
    } else if (uniforms.mode == MODE_UPSAMPLE_FIRST) {
        let upsampled_size = vec2<f32>(textureDimensions(input, i32(uniforms.lod + 1.0)));
        let upsampled_texture = upsample_tent9(input, samp, uniforms.lod + 1.0, tex_coords, 1.0 / upsampled_size, SAMPLE_SCALE);
  
        let existing = textureSampleLevel(input, samp, tex_coords, uniforms.lod).rgb;
        color.rgb = existing + upsampled_texture;
    } else if (uniforms.mode == MODE_UPSAMPLE) {
        let upsampled_size = vec2<f32>(textureDimensions(upsampled, i32(uniforms.lod + 1.0)));
        let upsampled_texture = upsample_tent9(upsampled, samp, uniforms.lod + 1.0, tex_coords, 1.0 / upsampled_size, SAMPLE_SCALE);

        let existing = textureSampleLevel(input, samp, tex_coords, uniforms.lod).rgb;
        color.rgb = existing + upsampled_texture;
    }

    textureStore(output, vec2<i32>(invocation), color);
}