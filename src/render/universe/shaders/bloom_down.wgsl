let EPISLON: f32 = 1.0e-4;

[[group(0), binding(0)]]
var output: texture_storage_2d<rgba16float, write>;

// The texture being filtered
[[group(0), binding(1)]]
var input: texture_2d<f32>;

[[group(0), binding(2)]]
var samp: sampler;

[[block]]
struct Uniforms {
    threshold: f32;
    knee: f32;
    lod: f32;
    filter: u32;
};

[[group(0), binding(3)]]
var<uniform> uniforms: Uniforms;

fn downsample_box13(t: texture_2d<f32>, s: sampler, lod: f32, uv: vec2<f32>, texel_size: vec2<f32>) -> vec3<f32> {
    // Center
    let a = textureSampleLevel(t, s, uv, lod).rgb;

    let texel_size = texel_size * 0.5; // Sample from center of texels
    
    // Inner box
    let b = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(-1.0, -1.0), lod).rgb;
    let c = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(-1.0, 1.0), lod).rgb;
    let d = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(1.0, 1.0), lod).rgb;
    let e = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(1.0, -1.0), lod).rgb;

    // Outer box
    let f = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(-2.0, -2.0), lod).rgb;
    let g = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(-2.0, 0.0), lod).rgb;
    let h = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(0.0, 2.0), lod).rgb;
    let i = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(2.0, 2.0), lod).rgb;
    let j = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(2.0, 2.0), lod).rgb;
    let k = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(2.0, 0.0), lod).rgb;
    let l = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(-2.0, -2.0), lod).rgb;
    let m = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(0.0, -2.0), lod).rgb;

    // Weights
    var result = vec3<f32>(0.0, 0.0, 0.0);
    // Inner box
    result = result + (b + c + d + e) * 0.5;
    // Bottom-left box
    result = result + (f + g + a + m) * 0.125;
    // Top-left box
    result = result + (g + h + i + a) * 0.125;
    // Top-right box
    result = result + (a + i + j + k) * 0.125;
    // Bottom-right box
    result = result + (m + a + k + l) * 0.125;

    // 4 samples each
    result = result * 0.25;

    return result;
}

// fn downsample_box4(t: texture_2d<f32>, s: sampler, lod: f32, uv: vec2<f32>, texel_size: vec2<f32>) -> vec3<f32> {
//     // Center
//     let a = textureSampleLevel(t, s, uv, lod).rgb;

//     let texel_size = texel_size * 0.5; // Sample from center of texels
    
//     // Inner box
//     let b = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(-1.0, -1.0), lod).rgb;
//     let c = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(-1.0, 1.0), lod).rgb;
//     let d = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(1.0, 1.0), lod).rgb;
//     let e = textureSampleLevel(t, s, uv + texel_size * vec2<f32>(1.0, -1.0), lod).rgb;

//     // Weights
//     var result = vec3<f32>(0.0, 0.0, 0.0);
//     result = result + (b + c + d + e);

//     // 4 samples each
//     result = result * 0.25;

//     return result;
// }

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

[[stage(compute), workgroup_size(4, 4, 1)]]
fn main([[builtin(global_invocation_id)]] invocation: vec3<u32>) {
    let output_size = vec2<f32>(textureDimensions(output));

    var coords = vec2<f32>(invocation.xy) / output_size;
    coords = coords + (1.0 / output_size) * 0.5;

    let input_size = vec2<f32>(textureDimensions(input, i32(uniforms.lod)));

    var color: vec4<f32>;
    color = vec4<f32>(downsample_box13(input, samp, uniforms.lod, coords, 1.0 / input_size), 1.0);
    if (uniforms.filter == 1u) {
        color = prefilter(color, coords);
    }

    color = vec4<f32>(color.rgb, 1.0);

    textureStore(output, vec2<i32>(invocation.xy), color);
}