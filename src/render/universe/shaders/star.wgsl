fn permute4(x: vec4<f32>) -> vec4<f32> { return ((x * 34. + 1.) * x) % vec4<f32>(289.); }
fn taylorInvSqrt4(r: vec4<f32>) -> vec4<f32> { return 1.79284291400159 - 0.85373472095314 * r; }

fn simplex_noise_3d(v: vec3<f32>) -> f32 {
  let C = vec2<f32>(1. / 6., 1. / 3.);
  let D = vec4<f32>(0., 0.5, 1., 2.);

  // First corner
  var i: vec3<f32>  = floor(v + dot(v, C.yyy));
  let x0 = v - i + dot(i, C.xxx);

  // Other corners
  let g = step(x0.yzx, x0.xyz);
  let l = 1.0 - g;
  let i1 = min(g.xyz, l.zxy);
  let i2 = max(g.xyz, l.zxy);

  // x0 = x0 - 0. + 0. * C
  let x1 = x0 - i1 + 1. * C.xxx;
  let x2 = x0 - i2 + 2. * C.xxx;
  let x3 = x0 - 1. + 3. * C.xxx;

  // Permutations
  i = i % vec3<f32>(289.);
  let p = permute4(permute4(permute4(
      i.z + vec4<f32>(0., i1.z, i2.z, 1. )) +
      i.y + vec4<f32>(0., i1.y, i2.y, 1. )) +
      i.x + vec4<f32>(0., i1.x, i2.x, 1. ));

  // Gradients (NxN points uniformly over a square, mapped onto an octahedron.)
  var n_: f32 = 1. / 7.; // N=7
  let ns = n_ * D.wyz - D.xzx;

  let j = p - 49. * floor(p * ns.z * ns.z); // mod(p, N*N)

  let x_ = floor(j * ns.z);
  let y_ = floor(j - 7.0 * x_); // mod(j, N)

  let x = x_ *ns.x + ns.yyyy;
  let y = y_ *ns.x + ns.yyyy;
  let h = 1.0 - abs(x) - abs(y);

  let b0 = vec4<f32>( x.xy, y.xy );
  let b1 = vec4<f32>( x.zw, y.zw );

  let s0 = floor(b0)*2.0 + 1.0;
  let s1 = floor(b1)*2.0 + 1.0;
  let sh = -step(h, vec4<f32>(0.));

  let a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
  let a1 = b1.xzyw + s1.xzyw*sh.zzww ;

  var p0: vec3<f32> = vec3<f32>(a0.xy, h.x);
  var p1: vec3<f32> = vec3<f32>(a0.zw, h.y);
  var p2: vec3<f32> = vec3<f32>(a1.xy, h.z);
  var p3: vec3<f32> = vec3<f32>(a1.zw, h.w);

  // Normalise gradients
  let norm = taylorInvSqrt4(vec4<f32>(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
  p0 = p0 * norm.x;
  p1 = p1 * norm.y;
  p2 = p2 * norm.z;
  p3 = p3 * norm.w;

  // Mix final noise value
  var m: vec4<f32> = 0.6 - vec4<f32>(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3));
  m = max(m, vec4<f32>(0.));
  m = m * m;
  return 42. * dot(m * m, vec4<f32>(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
}

fn noise(
    position: vec3<f32>,
    scale: f32,
    lacunarity: f32,
    gain: f32,
    octaves: f32
) -> f32 {
    var total = 0.0;
     
    var amplitude = 1.0;

    var pos = position * scale;

    var i = 0.0;

    loop {
        if (i > octaves) {
            break;
        }

        i = i + 1.0;

        total = total + amplitude * (simplex_noise_3d(pos) - 0.5);
        pos = pos * lacunarity;
        amplitude = amplitude *  gain;
    }
    
    return total;
}

struct VertexInput {
    [[location(0)]] pos: vec3<f32>;

    [[location(1)]] transform_matrix_0: vec4<f32>;
    [[location(2)]] transform_matrix_1: vec4<f32>;
    [[location(3)]] transform_matrix_2: vec4<f32>;
    [[location(4)]] transform_matrix_3: vec4<f32>;

    [[location(5)]] color: vec4<f32>;
    [[location(6)]] granules: vec4<f32>;
    [[location(7)]] sunspots: vec4<f32>;
};

struct VertexOutput {
    [[location(0)]] vert_pos: vec3<f32>;
    [[location(1)]] to_camera_vec: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
    [[location(3)]] granules: vec4<f32>;
    [[location(4)]] sunspots: vec4<f32>;
    [[builtin(position)]] clip_pos: vec4<f32>;
};

[[block]] struct Enviornment {
    proj_view: mat4x4<f32>;
    camera_pos: vec4<f32>;
    anim_time: f32;
};

[[group(0), binding(0)]] var<uniform> env: Enviornment;

[[stage(vertex)]]
fn vs_main(
    in: VertexInput,
) -> VertexOutput {

    let transform = mat4x4<f32>(
        in.transform_matrix_0,
        in.transform_matrix_1,
        in.transform_matrix_2,
        in.transform_matrix_3,
    );

    let world_pos = transform * vec4<f32>(in.pos, 1.0);

    var out: VertexOutput;
    out.vert_pos = in.pos;
    out.clip_pos = env.proj_view * world_pos;
    out.to_camera_vec = env.camera_pos.xyz - world_pos.xyz;
    out.color = in.color;
    out.granules = in.granules;
    out.sunspots = in.sunspots;

    return out;
}

// Fragment shader bindings

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    
    let scale = in.granules.x;
    let lacunarity = in.granules.y;
    let frequency = in.granules.z;
    let octaves = in.granules.w;

    let n = (noise(in.vert_pos, scale, lacunarity, frequency, octaves) + 1.0) + 0.5;

    // let n = (noise(in.vert_pos * 20.0, 40.0, 0.5, 3) + 1.0) * 0.5;

	// float n = (noise(position4D , 2, 40.0, 0.7) + 1.0) * 0.5;
	// //float total = n/1.5;

	//Sunspots
    let scale = in.sunspots.x;
    let offset = in.sunspots.y;
    let frequency = in.sunspots.z;
    let radius = in.sunspots.w;
    let t1 = simplex_noise_3d(in.vert_pos * frequency) - offset;
    let t2 = simplex_noise_3d((in.vert_pos + radius) * frequency) - offset;
    let ss = (max(t1, 0.0) * max(t2, 0.0)) * scale;
    let total = n - ss;

    return vec4<f32>(vec3<f32>(total * in.color.xyz), 1.0);

	// //Color
	// float u = (starTemperature - 800.0)/29200.0f;
	// vec4 starColour = texture(starTemperatureMap,vec2(u,1));

	// vec3 shiftedColour = useColourShift ? getTempColorShift(starTemperature) : vec3(0.0,0.0,0.0);

	// //float theta = dot(normalize(toCameraVector),pass_Position);
	// //out_Colour =  vec4(vec3(starColour*total) + vec3(shiftedColour*theta),1.0);

	// float theta = dot(normalize(toCameraVector),pass_Position);
	// out_Colour = vec4(vec3(starColour.xyz + shiftedColour*theta)*total,1.0);


	// out_Colour = mix(out_Colour,vec4(selectionColour,1.0),selectBlend*isSelected);

    // return vec4<f32>(vec3<f32>(simplex_noise_3d(in.vert_pos)), 1.0);
}