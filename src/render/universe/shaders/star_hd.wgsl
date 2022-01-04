// let ONE_DIV_289: f32 = 0.00346020761245674740484429065744;

// fn mod289(x: f32) -> f32 {
//     return x - floor(x * ONE_DIV_289) * 289.0;
// }

// fn mod289(x: vec2<f32>) -> vec2<f32> {
//     return x - floor(x * ONE_DIV_289) * 289.0;
// }

// fn mod289(x: vec3<f32>) -> vec3<f32> {
//     return x - floor(x * ONE_DIV_289) * 289.0;
// }

// fn mod289(x: vec4<f32>) -> vec4<f32>{
//     return x - floor(x * ONE_DIV_289) * 289.0;
// }

// // ( x*34.0 + 1.0 )*x = 
// // x*x*34.0 + x
// fn permute(x: f32) -> f32 {
// 	return mod289(
// 		x*x*34.0 + x
// 	);
// }

// fn permute(x: vec3<f32>) -> vec3<f32> {
// 	return mod289(
// 		x*x*34.0 + x
// 	);
// }

// fn permute(x: vec4<f32>) -> vec4<f32> {
// 	return mod289(
// 		x*x*34.0 + x
// 	);
// }

// fn grad4(j: f32, ip: vec4<f32>) -> vec4<f32>
// {
//     let ones = vec4<f32>(1.0, 1.0, 1.0, -1.0);

//     var p: vec4<f32>;
// 	p.xyz = floor(frac(j * ip.xyz) * 7.0) * ip.z - 1.0;
// 	p.w = 1.5 - dot( abs(p.xyz), ones.xyz );
	
// 	p.xyz = p.xyz - sign(p.xyz) * (p.w < 0);
	
// 	return p;
// }

// // ------ 2D ---------

// fn snoise(v: vec2<f32>) -> f32 {
//     let C = vec4<f32>(0.211324865405187, // (3.0-sqrt(3.0))/6.0
// 		0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
// 	 -0.577350269189626, // -1.0 + 2.0 * C.x
// 		0.024390243902439  // 1.0 / 41.0)
//     );

//     // First corner
// 	var i = floor( v + dot(v, C.yy) );
// 	let x0 = v - i + dot(i, C.xx);
	
//         // Other corners
// 	// float2 i1 = (x0.x > x0.y) ? float2(1.0, 0.0) : float2(0.0, 1.0);
// 	// Lex-DRL: afaik, step() in GPU is faster than if(), so:
// 	// step(x, y) = x <= y

//     var i1: vec2<f32>;

//     if (x0.x > x0.y) {
//         i1 = vec2<f32>(1.0, 0.0); 
//     }
//     else {
//         i1 = vec2<f32>(0.0, 1.0);
//     }

//     var x12 = x0.xyxy + C.xxzz;
// 	x12.xy -= i1;
	
//     // Permutations
// 	i = mod289(i); // Avoid truncation effects in permutation
// 	let p = permute(
// 		permute(
// 				i.y + vec3<f32>(0.0, i1.y, 1.0 )
// 		) + i.x + vec3<f32>(0.0, i1.x, 1.0 )
// 	);
	
// 	let m = max(
// 		0.5 - vec3<f32>(
// 			dot(x0, x0),
// 			dot(x12.xy, x12.xy),
// 			dot(x12.zw, x12.zw)
// 		),
// 		0.0
// 	);
// 	m = m*m;
// 	m = m*m;
	
//     // Gradients: 41 points uniformly over a line, mapped onto a diamond.
//     // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
	
// 	let x = 2.0 * frac(p * C.www) - 1.0;
// 	let h = abs(x) - 0.5;
// 	let ox = floor(x + 0.5);
// 	let a0 = x - ox;

//     // Normalise gradients implicitly by scaling m
//     // Approximation of: m *= inversesqrt( a0*a0 + h*h );
// 	m *= 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );

//     // Compute final noise value at P
// 	var g: vec3<f32>;
// 	g.x = a0.x * x0.x + h.x * x0.y;
// 	g.yz = a0.yz * x12.xz + h.yz * x12.yw;
// 	return 130.0 * dot(m, g);
// }

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
    out.uv = vertex;
    out.pos = vec4<f32>(vertex, 0.0, 1.0);
    return out;
}

// Signed distance function for a sphere of radius r
fn distance_from_sphere(p: vec3<f32>, c: vec3<f32>, r: f32) -> f32 {
    return length(p - c) - r;
}

fn distance_function(pos: vec3<f32>) -> f32 {
    return distance_from_sphere(pos, vec3<f32>(0.0, 0.0, 5.0), 1.0);
}

struct FragmentOutput {
    [[builtin(frag_depth)]] depth: f32;
    [[location(0)]] color: vec4<f32>;
};

[[block]] 
struct Enviornment {
    clip_to_world: mat4x4<f32>;
    world_to_clip: mat4x4<f32>;
    camera: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> env: Enviornment;

let NUMBER_OF_STEPS: i32 = 32;
let MIN_HIT_DISTANCE: f32 = 0.001;
let MAX_TRACE_DISTANCE: f32 = 1000.0;

[[stage(fragment)]]
fn fs_main(in: VertexData) -> FragmentOutput {

    // Setup

    // Origin of the rays
    let origin = env.camera.xyz;
    // Clip space coordinates of this fragment
    let frag_clip = in.uv;

    let frag_world = (env.clip_to_world * vec4<f32>(frag_clip, env.camera.w, 1.0)).xyz;
    // Find world space coordinates of this fragment, subtract the origin, and normalize to get direction vector.
    let direction = normalize(frag_world - origin);    

    // Output

    var out: FragmentOutput;
    out.color = vec4<f32>(direction, 1.0);
    out.depth = 1.0;

    // Ray marching

    var total_distance: f32 = 0.0;


    loop {
        let current_pos = origin + total_distance * direction;
        let distance_to_closest = distance_function(current_pos);

        if (distance_to_closest < MIN_HIT_DISTANCE) {
            let world_pos = origin + distance_to_closest * direction;

            let clip_pos = env.world_to_clip * vec4<f32>(world_pos, 1.0);

            out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
            out.depth = clip_pos.z / clip_pos.w;
                

            break;
        } 

        if (total_distance > MAX_TRACE_DISTANCE) {
            discard;
        }

        total_distance = total_distance + distance_to_closest;
    }

    return out;
}