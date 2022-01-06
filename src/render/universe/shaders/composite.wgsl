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
    out.uv = (vertex + 1.0) * 0.5;
    out.pos = vec4<f32>(vertex, 0.0, 1.0);
    return out;
}

[[group(0), binding(0)]]
var<uniform> texture: texture_2d<f32>;

[[group(0), binding(1))]]
var<uniform> bloom_texture: texture_2d<f32>;

// [[group(0), binding(2)]]
// var<uniform> bloom_dirt_texture: texture2d<f32>;

[[group(0), binding(2)]]
var<uniform> default_sampler: sampler;

[[block]]
struct Settings {
    exposure: f32;
    bloom_intensity: f32;
    bloom_dirt_intensity: f32;
}

[[group(0), binding(3)]]
var<uniform> settings: Settings;

fn upsampleTent9(t: texture_2d<f32>, s: sampler, lod: f32, uv: vec2<f32>, texelSize: vec2<f32>, radius: f32) -> vec3<f32>
{
	let offset = texelSize.xyxy * vec4<f32>(1.0, 1.0, -1.0, 0.0) * radius;

	// Center
	var result: vec3<f32> = textureSampleLevel(t, s, uv, lod).rgb * 4.0;

	result += textureSampleLevel(t, s, uv - offset.xy, lod).rgb;
	result += textureSampleLevel(t, s, uv - offset.wy, lod).rgb * 2.0;
	result += textureSampleLevel(t, s, uv - offset.zy, lod).rgb;

	result += textureSampleLevel(t, s, uv + offset.zw, lod).rgb * 2.0;
	result += textureSampleLevel(t, s, uv + offset.xw, lod).rgb * 2.0;

	result += textureSampleLevel(t, s, uv + offset.zy, lod).rgb;
	result += textureSampleLevel(t, s, uv + offset.wy, lod).rgb * 2.0;
	result += textureSampleLevel(t, s, uv + offset.xy, lod).rgb;

	return result * (1.0 / 16.0);
}

// Based on http://www.oscars.org/science-technology/sci-tech-projects/aces
fn ACESTonemap(color: vec3<f32>) -> vec3<f32>
{
	let m1 = mat3x3<f32>(
		0.59719, 0.07600, 0.02840,
		0.35458, 0.90834, 0.13383,
		0.04823, 0.01566, 0.83777
	);
	let m2 = mat3x3<f32>(
		1.60475, -0.10208, -0.00327,
		-0.53108, 1.10813, -0.07276,
		-0.07367, -0.00605, 1.07602
	);
	let v = m1 * color;
	let a = v * (v + 0.0245786) - 0.000090537;
	let b = v * (0.983729 * v + 0.4329510) + 0.238081;
	return clamp(m2 * (a / b), 0.0, 1.0);
}

vec3 GammaCorrect(vec3 color, float gamma)
{
	return pow(color, vec3(1.0 / gamma));
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

    let texSize = textureDimensions(bloom_texture, 0);
    let fTexSize = vec2<f32>(texSize);

    let bloom = upsample_tent9(bloom_texture, default_sampler, in.uv, 1.0 / fTexSize, SAMPLE_SCALE) * settings.bloom_intensity;
    //let bloom_dirt = textureSample(bloom_texture, in.uv).rgb * settings.bloom_dirt_intensity;

    var color = textureSample(texture);
    color += bloom;
    // color += bloom * bloom_dirt;
    color *= settings.exposure;
	color = ACESTonemap(color);
	color = GammaCorrect(color.rgb, gamma);
	out.color = vec4(color, 1.0);
}