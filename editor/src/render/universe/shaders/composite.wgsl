var<private> vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
);

struct VertexInput {
    [[builtin(vertex_index)]] vertex_index: u32;
};

struct VertexData {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] pos: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexData {
    let vertex = vertices[in.vertex_index];
    var out: VertexData;
    out.uv = (vec2<f32>(vertex.x, -vertex.y) + 1.0) * 0.5;
    out.pos = vec4<f32>(vertex, 0.0, 1.0);
    return out;
}

[[group(0), binding(0)]]
var scene: texture_2d<f32>;

[[group(0), binding(1)]]
var bloom: texture_2d<f32>;

// [[group(0), binding(2)]]
// var<uniform> bloom_dirt_texture: texture2d<f32>;

[[group(0), binding(2)]]
var samp: sampler;

struct Settings {
    exposure: f32;
    bloom_intensity: f32;
    bloom_dirt_intensity: f32;
};

[[group(0), binding(3)]]
var<uniform> settings: Settings;

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

// Based on http://www.oscars.org/science-technology/sci-tech-projects/aces
fn ACESTonemap(color: vec3<f32>) -> vec3<f32>
{
	let m1 = mat3x3<f32>(
		vec3<f32>(0.59719, 0.07600, 0.02840),
		vec3<f32>(0.35458, 0.90834, 0.13383),
		vec3<f32>(0.04823, 0.01566, 0.83777)
	);
	let m2 = mat3x3<f32>(
		vec3<f32>(1.60475, -0.10208, -0.00327),
		vec3<f32>(-0.53108, 1.10813, -0.07276),
		vec3<f32>(-0.07367, -0.00605, 1.07602)
	);
	let v = m1 * color;
	let a = v * (v + 0.0245786) - 0.000090537;
	let b = v * (0.983729 * v + 0.4329510) + 0.238081;
	return clamp(m2 * (a / b), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn GammaCorrect(color: vec3<f32>, gamma: f32) -> vec3<f32>
{
	return pow(color, vec3<f32>(1.0 / gamma));
}

struct FragmentOutput {
    [[location(0)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexData) -> FragmentOutput {
    var out: FragmentOutput;

    let GAMMA: f32 = 2.2;
    let PURE_WHITE = 1.0;
    let SAMPLE_SCALE: f32 = 0.5;

    let texSize = textureDimensions(bloom, 0);
    let fTexSize = vec2<f32>(texSize);

    let bloom_color = upsample_tent9(bloom, samp, 0.0, in.uv, 1.0 / fTexSize, SAMPLE_SCALE) * settings.bloom_intensity;
    //let bloom_dirt = textureSample(bloom_texture, in.uv).rgb * settings.bloom_dirt_intensity;

    var color = textureSample(scene, samp, in.uv).rgb;
    color = color + bloom_color;
    // color += bloom * bloom_dirt;
    color = color * settings.exposure;
	color = ACESTonemap(color);
	color = GammaCorrect(color, GAMMA);
	out.color = vec4<f32>(color, 1.0);

    return out;
}