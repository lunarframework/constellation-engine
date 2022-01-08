//	Simplex 4D Noise 
//	by Ian McEwan, Ashima Arts
//
fn permute(x: vec4<f32>) -> vec4<f32> { 
    return (((x*34.0)+1.0)*x) % 289.0;
}
fn permute1(x: f32) -> f32 { return floor((((x*34.0)+1.0)*x) % 289.0);}
fn taylor_inv_sqrt(r: vec4<f32>) -> vec4<f32> { return 1.79284291400159 - 0.85373472095314 * r;}
fn taylor_inv_sqrt1(r: f32) -> f32 { return 1.79284291400159 - 0.85373472095314 * r;}

fn frac(x: vec3<f32>) -> vec3<f32> {
    return x - trunc(x);
}

fn grad4(j: f32, ip: vec4<f32>) -> vec4<f32> {
    var j1: f32;
    var ip1: vec4<f32>;
    var ones: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, -1.0);
    var p: vec4<f32>;
    var s: vec4<f32>;

    j1 = j;
    ip1 = ip;
    let e13: vec4<f32> = p;
    let e15: f32 = j1;
    let e17: vec4<f32> = ip1;
    let e20: f32 = j1;
    let e22: vec4<f32> = ip1;
    let e28: f32 = j1;
    let e30: vec4<f32> = ip1;
    let e33: f32 = j1;
    let e35: vec4<f32> = ip1;
    let e42: vec4<f32> = ip1;
    let e47: vec3<f32> = ((floor((fract((vec3<f32>(e33) * e35.xyz)) * 7.0)) * e42.z) - vec3<f32>(1.0));
    p.x = e47.x;
    p.y = e47.y;
    p.z = e47.z;
    let e56: vec4<f32> = p;
    let e58: vec4<f32> = p;
    let e61: vec4<f32> = ones;
    let e63: vec4<f32> = p;
    let e65: vec4<f32> = p;
    let e68: vec4<f32> = ones;
    p.w = (1.5 - dot(abs(e65.xyz), e68.xyz));
    let e75: vec4<f32> = p;
    s = select(vec4<f32>(0.0), vec4<f32>(1.0), (e75 < vec4<f32>(0.0)));
    let e84: vec4<f32> = p;
    let e86: vec4<f32> = p;
    let e88: vec4<f32> = s;
    let e95: vec4<f32> = s;
    let e98: vec3<f32> = (e86.xyz + (((e88.xyz * 2.0) - vec3<f32>(1.0)) * e95.www));
    p.x = e98.x;
    p.y = e98.y;
    p.z = e98.z;
    let e105: vec4<f32> = p;
    return e105;
}

// fn snoise(v: vec4<f32>) -> f32 {
//     let C = vec2<f32>( 0.138196601125010504,  // (5 - sqrt(5))/20  G4
//                             0.309016994374947451); // (sqrt(5) - 1)/4   F4
//     // First corner
//     let i  = floor(v + dot(v, C.yyyy) );
//     let x0 = v - i + dot(i, C.xxxx);

//     // Other corners

//     // Rank sorting originally contributed by Bill Licea-Kane, AMD (formerly ATI)
//   vec4 i0;

//   let is_x = vec3<f32>(x0.xxx >= x0.yzw);
//   let is_yz = vec3<f32>(x0.yyz >= x0.zww);

//   var i0: vec3<f32>;

//   i0.x = is_x.x + is_x.y + is_x.z;
//   i0.yzw = 1.0 - isX;

  

// //  i0.y += dot( isYZ.xy, vec2( 1.0 ) );
//   i0.y += isYZ.x + isYZ.y;
//   i0.zw += 1.0 - isYZ.xy;

//   i0.z += isYZ.z;
//   i0.w += 1.0 - isYZ.z;

//   // i0 now contains the unique values 0,1,2,3 in each channel
//   vec4 i3 = clamp( i0, 0.0, 1.0 );
//   vec4 i2 = clamp( i0-1.0, 0.0, 1.0 );
//   vec4 i1 = clamp( i0-2.0, 0.0, 1.0 );

//   //  x0 = x0 - 0.0 + 0.0 * C 
//   vec4 x1 = x0 - i1 + 1.0 * C.xxxx;
//   vec4 x2 = x0 - i2 + 2.0 * C.xxxx;
//   vec4 x3 = x0 - i3 + 3.0 * C.xxxx;
//   vec4 x4 = x0 - 1.0 + 4.0 * C.xxxx;

// // Permutations
//   let i = mod(i, 289.0); 
//   float j0 = permute( permute( permute( permute(i.w) + i.z) + i.y) + i.x);
//   vec4 j1 = permute( permute( permute( permute (
//              i.w + vec4(i1.w, i2.w, i3.w, 1.0 ))
//            + i.z + vec4(i1.z, i2.z, i3.z, 1.0 ))
//            + i.y + vec4(i1.y, i2.y, i3.y, 1.0 ))
//            + i.x + vec4(i1.x, i2.x, i3.x, 1.0 ));
// // Gradients
// // ( 7*7*6 points uniformly over a cube, mapped onto a 4-octahedron.)
// // 7*7*6 = 294, which is close to the ring size 17*17 = 289.

//   vec4 ip = vec4(1.0/294.0, 1.0/49.0, 1.0/7.0, 0.0) ;

//   vec4 p0 = grad4(j0,   ip);
//   vec4 p1 = grad4(j1.x, ip);
//   vec4 p2 = grad4(j1.y, ip);
//   vec4 p3 = grad4(j1.z, ip);
//   vec4 p4 = grad4(j1.w, ip);

// // Normalise gradients
//   vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
//   p0 *= norm.x;
//   p1 *= norm.y;
//   p2 *= norm.z;
//   p3 *= norm.w;
//   p4 *= taylorInvSqrt(dot(p4,p4));

// // Mix contributions from the five corners
//   vec3 m0 = max(0.6 - vec3(dot(x0,x0), dot(x1,x1), dot(x2,x2)), 0.0);
//   vec2 m1 = max(0.6 - vec2(dot(x3,x3), dot(x4,x4)            ), 0.0);
//   m0 = m0 * m0;
//   m1 = m1 * m1;
//   return 49.0 * ( dot(m0*m0, vec3( dot( p0, x0 ), dot( p1, x1 ), dot( p2, x2 )))
//                + dot(m1*m1, vec2( dot( p3, x3 ), dot( p4, x4 ) ) ) ) ;

// }

fn snoise(v: vec4<f32>) -> f32 {
    var v1: vec4<f32>;
    var C: vec2<f32> = vec2<f32>(0.13819660246372223, 0.30901700258255005);
    var i: vec4<f32>;
    var x0_: vec4<f32>;
    var i0_: vec4<f32>;
    var isX: vec3<f32>;
    var isYZ: vec3<f32>;
    var i3_: vec4<f32>;
    var i2_: vec4<f32>;
    var i1_: vec4<f32>;
    var x1_: vec4<f32>;
    var x2_: vec4<f32>;
    var x3_: vec4<f32>;
    var x4_: vec4<f32>;
    var j0_: f32;
    var j1_: vec4<f32>;
    var ip2: vec4<f32> = vec4<f32>(0.003401360544217687, 0.02040816326530612, 0.14285714285714285, 0.0);
    var p0_: vec4<f32>;
    var p1_: vec4<f32>;
    var p2_: vec4<f32>;
    var p3_: vec4<f32>;
    var p4_: vec4<f32>;
    var norm: vec4<f32>;
    var m0_: vec3<f32>;
    var m1_: vec2<f32>;

    v1 = v;
    let e6: vec4<f32> = v1;
    let e8: vec2<f32> = C;
    let e10: vec4<f32> = v1;
    let e11: vec2<f32> = C;
    let e16: vec4<f32> = v1;
    let e18: vec2<f32> = C;
    let e20: vec4<f32> = v1;
    let e21: vec2<f32> = C;
    i = floor((e16 + vec4<f32>(dot(e20, e21.yyyy))));
    let e28: vec4<f32> = v1;
    let e29: vec4<f32> = i;
    let e32: vec2<f32> = C;
    let e34: vec4<f32> = i;
    let e35: vec2<f32> = C;
    x0_ = ((e28 - e29) + vec4<f32>(dot(e34, e35.xxxx)));
    let e42: vec4<f32> = x0_;
    let e44: vec4<f32> = x0_;
    let e46: vec4<f32> = x0_;
    let e48: vec4<f32> = x0_;
    isX = step(e46.yzw, e48.xxx);
    let e52: vec4<f32> = x0_;
    let e54: vec4<f32> = x0_;
    let e56: vec4<f32> = x0_;
    let e58: vec4<f32> = x0_;
    isYZ = step(e56.zww, e58.yyz);
    let e63: vec3<f32> = isX;
    let e65: vec3<f32> = isX;
    let e68: vec3<f32> = isX;
    i0_.x = ((e63.x + e65.y) + e68.z);
    let e71: vec4<f32> = i0_;
    let e74: vec3<f32> = isX;
    let e76: vec3<f32> = (vec3<f32>(1.0) - e74);
    i0_.y = e76.x;
    i0_.z = e76.y;
    i0_.w = e76.z;
    let e84: vec4<f32> = i0_;
    let e86: vec3<f32> = isYZ;
    let e88: vec3<f32> = isYZ;
    i0_.y = (e84.y + (e86.x + e88.y));
    let e92: vec4<f32> = i0_;
    let e94: vec4<f32> = i0_;
    let e97: vec3<f32> = isYZ;
    let e101: vec2<f32> = (e94.zw + (vec2<f32>(1.0) - e97.xy));
    i0_.z = e101.x;
    i0_.w = e101.y;
    let e107: vec4<f32> = i0_;
    let e109: vec3<f32> = isYZ;
    i0_.z = (e107.z + e109.z);
    let e113: vec4<f32> = i0_;
    let e116: vec3<f32> = isYZ;
    i0_.w = (e113.w + (1.0 - e116.z));
    let e123: vec4<f32> = i0_;
    i3_ = clamp(e123, vec4<f32>(0.0), vec4<f32>(1.0));
    let e130: vec4<f32> = i0_;
    let e136: vec4<f32> = i0_;
    i2_ = clamp((e136 - vec4<f32>(1.0)), vec4<f32>(0.0), vec4<f32>(1.0));
    let e146: vec4<f32> = i0_;
    let e152: vec4<f32> = i0_;
    i1_ = clamp((e152 - vec4<f32>(2.0)), vec4<f32>(0.0), vec4<f32>(1.0));
    let e162: vec4<f32> = x0_;
    let e163: vec4<f32> = i1_;
    let e166: vec2<f32> = C;
    x1_ = ((e162 - e163) + (1.0 * e166.xxxx));
    let e171: vec4<f32> = x0_;
    let e172: vec4<f32> = i2_;
    let e175: vec2<f32> = C;
    x2_ = ((e171 - e172) + (2.0 * e175.xxxx));
    let e180: vec4<f32> = x0_;
    let e181: vec4<f32> = i3_;
    let e184: vec2<f32> = C;
    x3_ = ((e180 - e181) + (3.0 * e184.xxxx));
    let e189: vec4<f32> = x0_;
    let e194: vec2<f32> = C;
    x4_ = ((e189 - vec4<f32>(1.0)) + (4.0 * e194.xxxx));
    let e201: vec4<f32> = i;
    i = (e201 % vec4<f32>(289.0));
    let e205: vec4<f32> = i;
    let e207: vec4<f32> = i;
    let e209: f32 = permute1(e207.w);
    let e210: vec4<f32> = i;
    let e213: vec4<f32> = i;
    let e215: vec4<f32> = i;
    let e217: f32 = permute1(e215.w);
    let e218: vec4<f32> = i;
    let e221: f32 = permute1((e217 + e218.z));
    let e222: vec4<f32> = i;
    let e225: vec4<f32> = i;
    let e227: vec4<f32> = i;
    let e229: f32 = permute1(e227.w);
    let e230: vec4<f32> = i;
    let e233: vec4<f32> = i;
    let e235: vec4<f32> = i;
    let e237: f32 = permute1(e235.w);
    let e238: vec4<f32> = i;
    let e241: f32 = permute1((e237 + e238.z));
    let e242: vec4<f32> = i;
    let e245: f32 = permute1((e241 + e242.y));
    let e246: vec4<f32> = i;
    let e249: vec4<f32> = i;
    let e251: vec4<f32> = i;
    let e253: f32 = permute1(e251.w);
    let e254: vec4<f32> = i;
    let e257: vec4<f32> = i;
    let e259: vec4<f32> = i;
    let e261: f32 = permute1(e259.w);
    let e262: vec4<f32> = i;
    let e265: f32 = permute1((e261 + e262.z));
    let e266: vec4<f32> = i;
    let e269: vec4<f32> = i;
    let e271: vec4<f32> = i;
    let e273: f32 = permute1(e271.w);
    let e274: vec4<f32> = i;
    let e277: vec4<f32> = i;
    let e279: vec4<f32> = i;
    let e281: f32 = permute1(e279.w);
    let e282: vec4<f32> = i;
    let e285: f32 = permute1((e281 + e282.z));
    let e286: vec4<f32> = i;
    let e289: f32 = permute1((e285 + e286.y));
    let e290: vec4<f32> = i;
    let e293: f32 = permute1((e289 + e290.x));
    j0_ = e293;
    let e295: vec4<f32> = i;
    let e297: vec4<f32> = i1_;
    let e299: vec4<f32> = i2_;
    let e301: vec4<f32> = i3_;
    let e307: vec4<f32> = i;
    let e309: vec4<f32> = i1_;
    let e311: vec4<f32> = i2_;
    let e313: vec4<f32> = i3_;
    let e319: vec4<f32> = permute((vec4<f32>(e307.w) + vec4<f32>(e309.w, e311.w, e313.w, 1.0)));
    let e320: vec4<f32> = i;
    let e324: vec4<f32> = i1_;
    let e326: vec4<f32> = i2_;
    let e328: vec4<f32> = i3_;
    let e333: vec4<f32> = i;
    let e335: vec4<f32> = i1_;
    let e337: vec4<f32> = i2_;
    let e339: vec4<f32> = i3_;
    let e345: vec4<f32> = i;
    let e347: vec4<f32> = i1_;
    let e349: vec4<f32> = i2_;
    let e351: vec4<f32> = i3_;
    let e357: vec4<f32> = permute((vec4<f32>(e345.w) + vec4<f32>(e347.w, e349.w, e351.w, 1.0)));
    let e358: vec4<f32> = i;
    let e362: vec4<f32> = i1_;
    let e364: vec4<f32> = i2_;
    let e366: vec4<f32> = i3_;
    let e371: vec4<f32> = permute(((e357 + vec4<f32>(e358.z)) + vec4<f32>(e362.z, e364.z, e366.z, 1.0)));
    let e372: vec4<f32> = i;
    let e376: vec4<f32> = i1_;
    let e378: vec4<f32> = i2_;
    let e380: vec4<f32> = i3_;
    let e385: vec4<f32> = i;
    let e387: vec4<f32> = i1_;
    let e389: vec4<f32> = i2_;
    let e391: vec4<f32> = i3_;
    let e397: vec4<f32> = i;
    let e399: vec4<f32> = i1_;
    let e401: vec4<f32> = i2_;
    let e403: vec4<f32> = i3_;
    let e409: vec4<f32> = permute((vec4<f32>(e397.w) + vec4<f32>(e399.w, e401.w, e403.w, 1.0)));
    let e410: vec4<f32> = i;
    let e414: vec4<f32> = i1_;
    let e416: vec4<f32> = i2_;
    let e418: vec4<f32> = i3_;
    let e423: vec4<f32> = i;
    let e425: vec4<f32> = i1_;
    let e427: vec4<f32> = i2_;
    let e429: vec4<f32> = i3_;
    let e435: vec4<f32> = i;
    let e437: vec4<f32> = i1_;
    let e439: vec4<f32> = i2_;
    let e441: vec4<f32> = i3_;
    let e447: vec4<f32> = permute((vec4<f32>(e435.w) + vec4<f32>(e437.w, e439.w, e441.w, 1.0)));
    let e448: vec4<f32> = i;
    let e452: vec4<f32> = i1_;
    let e454: vec4<f32> = i2_;
    let e456: vec4<f32> = i3_;
    let e461: vec4<f32> = permute(((e447 + vec4<f32>(e448.z)) + vec4<f32>(e452.z, e454.z, e456.z, 1.0)));
    let e462: vec4<f32> = i;
    let e466: vec4<f32> = i1_;
    let e468: vec4<f32> = i2_;
    let e470: vec4<f32> = i3_;
    let e475: vec4<f32> = permute(((e461 + vec4<f32>(e462.y)) + vec4<f32>(e466.y, e468.y, e470.y, 1.0)));
    let e476: vec4<f32> = i;
    let e480: vec4<f32> = i1_;
    let e482: vec4<f32> = i2_;
    let e484: vec4<f32> = i3_;
    let e489: vec4<f32> = i;
    let e491: vec4<f32> = i1_;
    let e493: vec4<f32> = i2_;
    let e495: vec4<f32> = i3_;
    let e501: vec4<f32> = i;
    let e503: vec4<f32> = i1_;
    let e505: vec4<f32> = i2_;
    let e507: vec4<f32> = i3_;
    let e513: vec4<f32> = permute((vec4<f32>(e501.w) + vec4<f32>(e503.w, e505.w, e507.w, 1.0)));
    let e514: vec4<f32> = i;
    let e518: vec4<f32> = i1_;
    let e520: vec4<f32> = i2_;
    let e522: vec4<f32> = i3_;
    let e527: vec4<f32> = i;
    let e529: vec4<f32> = i1_;
    let e531: vec4<f32> = i2_;
    let e533: vec4<f32> = i3_;
    let e539: vec4<f32> = i;
    let e541: vec4<f32> = i1_;
    let e543: vec4<f32> = i2_;
    let e545: vec4<f32> = i3_;
    let e551: vec4<f32> = permute((vec4<f32>(e539.w) + vec4<f32>(e541.w, e543.w, e545.w, 1.0)));
    let e552: vec4<f32> = i;
    let e556: vec4<f32> = i1_;
    let e558: vec4<f32> = i2_;
    let e560: vec4<f32> = i3_;
    let e565: vec4<f32> = permute(((e551 + vec4<f32>(e552.z)) + vec4<f32>(e556.z, e558.z, e560.z, 1.0)));
    let e566: vec4<f32> = i;
    let e570: vec4<f32> = i1_;
    let e572: vec4<f32> = i2_;
    let e574: vec4<f32> = i3_;
    let e579: vec4<f32> = i;
    let e581: vec4<f32> = i1_;
    let e583: vec4<f32> = i2_;
    let e585: vec4<f32> = i3_;
    let e591: vec4<f32> = i;
    let e593: vec4<f32> = i1_;
    let e595: vec4<f32> = i2_;
    let e597: vec4<f32> = i3_;
    let e603: vec4<f32> = permute((vec4<f32>(e591.w) + vec4<f32>(e593.w, e595.w, e597.w, 1.0)));
    let e604: vec4<f32> = i;
    let e608: vec4<f32> = i1_;
    let e610: vec4<f32> = i2_;
    let e612: vec4<f32> = i3_;
    let e617: vec4<f32> = i;
    let e619: vec4<f32> = i1_;
    let e621: vec4<f32> = i2_;
    let e623: vec4<f32> = i3_;
    let e629: vec4<f32> = i;
    let e631: vec4<f32> = i1_;
    let e633: vec4<f32> = i2_;
    let e635: vec4<f32> = i3_;
    let e641: vec4<f32> = permute((vec4<f32>(e629.w) + vec4<f32>(e631.w, e633.w, e635.w, 1.0)));
    let e642: vec4<f32> = i;
    let e646: vec4<f32> = i1_;
    let e648: vec4<f32> = i2_;
    let e650: vec4<f32> = i3_;
    let e655: vec4<f32> = permute(((e641 + vec4<f32>(e642.z)) + vec4<f32>(e646.z, e648.z, e650.z, 1.0)));
    let e656: vec4<f32> = i;
    let e660: vec4<f32> = i1_;
    let e662: vec4<f32> = i2_;
    let e664: vec4<f32> = i3_;
    let e669: vec4<f32> = permute(((e655 + vec4<f32>(e656.y)) + vec4<f32>(e660.y, e662.y, e664.y, 1.0)));
    let e670: vec4<f32> = i;
    let e674: vec4<f32> = i1_;
    let e676: vec4<f32> = i2_;
    let e678: vec4<f32> = i3_;
    let e683: vec4<f32> = permute(((e669 + vec4<f32>(e670.x)) + vec4<f32>(e674.x, e676.x, e678.x, 1.0)));
    j1_ = e683;
    let e699: f32 = j0_;
    let e700: vec4<f32> = ip2;
    let e701: vec4<f32> = grad4(e699, e700);
    p0_ = e701;
    let e703: vec4<f32> = j1_;
    let e706: vec4<f32> = j1_;
    let e708: vec4<f32> = ip2;
    let e709: vec4<f32> = grad4(e706.x, e708);
    p1_ = e709;
    let e711: vec4<f32> = j1_;
    let e714: vec4<f32> = j1_;
    let e716: vec4<f32> = ip2;
    let e717: vec4<f32> = grad4(e714.y, e716);
    p2_ = e717;
    let e719: vec4<f32> = j1_;
    let e722: vec4<f32> = j1_;
    let e724: vec4<f32> = ip2;
    let e725: vec4<f32> = grad4(e722.z, e724);
    p3_ = e725;
    let e727: vec4<f32> = j1_;
    let e730: vec4<f32> = j1_;
    let e732: vec4<f32> = ip2;
    let e733: vec4<f32> = grad4(e730.w, e732);
    p4_ = e733;
    let e737: vec4<f32> = p0_;
    let e738: vec4<f32> = p0_;
    let e742: vec4<f32> = p1_;
    let e743: vec4<f32> = p1_;
    let e747: vec4<f32> = p2_;
    let e748: vec4<f32> = p2_;
    let e752: vec4<f32> = p3_;
    let e753: vec4<f32> = p3_;
    let e758: vec4<f32> = p0_;
    let e759: vec4<f32> = p0_;
    let e763: vec4<f32> = p1_;
    let e764: vec4<f32> = p1_;
    let e768: vec4<f32> = p2_;
    let e769: vec4<f32> = p2_;
    let e773: vec4<f32> = p3_;
    let e774: vec4<f32> = p3_;
    let e777: vec4<f32> = taylor_inv_sqrt(vec4<f32>(dot(e758, e759), dot(e763, e764), dot(e768, e769), dot(e773, e774)));
    norm = e777;
    let e779: vec4<f32> = p0_;
    let e780: vec4<f32> = norm;
    p0_ = (e779 * e780.x);
    let e783: vec4<f32> = p1_;
    let e784: vec4<f32> = norm;
    p1_ = (e783 * e784.y);
    let e787: vec4<f32> = p2_;
    let e788: vec4<f32> = norm;
    p2_ = (e787 * e788.z);
    let e791: vec4<f32> = p3_;
    let e792: vec4<f32> = norm;
    p3_ = (e791 * e792.w);
    let e795: vec4<f32> = p4_;
    let e798: vec4<f32> = p4_;
    let e799: vec4<f32> = p4_;
    let e803: vec4<f32> = p4_;
    let e804: vec4<f32> = p4_;
    let e806: f32 = taylor_inv_sqrt1(dot(e803, e804));
    p4_ = (e795 * e806);
    let e811: vec4<f32> = x0_;
    let e812: vec4<f32> = x0_;
    let e816: vec4<f32> = x1_;
    let e817: vec4<f32> = x1_;
    let e821: vec4<f32> = x2_;
    let e822: vec4<f32> = x2_;
    let e831: vec4<f32> = x0_;
    let e832: vec4<f32> = x0_;
    let e836: vec4<f32> = x1_;
    let e837: vec4<f32> = x1_;
    let e841: vec4<f32> = x2_;
    let e842: vec4<f32> = x2_;
    m0_ = max((vec3<f32>(0.6000000238418579) - vec3<f32>(dot(e831, e832), dot(e836, e837), dot(e841, e842))), vec3<f32>(0.0));
    let e854: vec4<f32> = x3_;
    let e855: vec4<f32> = x3_;
    let e859: vec4<f32> = x4_;
    let e860: vec4<f32> = x4_;
    let e869: vec4<f32> = x3_;
    let e870: vec4<f32> = x3_;
    let e874: vec4<f32> = x4_;
    let e875: vec4<f32> = x4_;
    m1_ = max((vec2<f32>(0.6000000238418579) - vec2<f32>(dot(e869, e870), dot(e874, e875))), vec2<f32>(0.0));
    let e884: vec3<f32> = m0_;
    let e885: vec3<f32> = m0_;
    m0_ = (e884 * e885);
    let e887: vec2<f32> = m1_;
    let e888: vec2<f32> = m1_;
    m1_ = (e887 * e888);
    let e891: vec3<f32> = m0_;
    let e892: vec3<f32> = m0_;
    let e896: vec4<f32> = p0_;
    let e897: vec4<f32> = x0_;
    let e901: vec4<f32> = p1_;
    let e902: vec4<f32> = x1_;
    let e906: vec4<f32> = p2_;
    let e907: vec4<f32> = x2_;
    let e910: vec3<f32> = m0_;
    let e911: vec3<f32> = m0_;
    let e915: vec4<f32> = p0_;
    let e916: vec4<f32> = x0_;
    let e920: vec4<f32> = p1_;
    let e921: vec4<f32> = x1_;
    let e925: vec4<f32> = p2_;
    let e926: vec4<f32> = x2_;
    let e930: vec2<f32> = m1_;
    let e931: vec2<f32> = m1_;
    let e935: vec4<f32> = p3_;
    let e936: vec4<f32> = x3_;
    let e940: vec4<f32> = p4_;
    let e941: vec4<f32> = x4_;
    let e944: vec2<f32> = m1_;
    let e945: vec2<f32> = m1_;
    let e949: vec4<f32> = p3_;
    let e950: vec4<f32> = x3_;
    let e954: vec4<f32> = p4_;
    let e955: vec4<f32> = x4_;
    return (49.0 * (dot((e910 * e911), vec3<f32>(dot(e915, e916), dot(e920, e921), dot(e925, e926))) + dot((e944 * e945), vec2<f32>(dot(e949, e950), dot(e954, e955)))));
}


fn noise(
    position: vec4<f32>,
    lacunarity: f32,
    gain: f32,
    octaves: f32
) -> f32 {
    var total = 0.0;
     
    var amplitude = 1.0;

    var pos = position;

    var i = 0.0;

    loop {
        if (i > octaves) {
            break;
        }

        i = i + 1.0;

        total = total + amplitude * snoise(pos);
        pos = pos * lacunarity;
        amplitude = amplitude * gain;
    }
    
    return total;
}

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

[[block]] 
struct Enviornment {
    clip_to_world: mat4x4<f32>;
    world_to_clip: mat4x4<f32>;
    camera: vec4<f32>;
    time: f32;
};

[[group(0), binding(0)]]
var<uniform> env: Enviornment;

[[block]] 
struct Star {
    pos: vec4<f32>;
    color: vec4<f32>;
    shift: vec4<f32>;
    granule_lacunarity: f32;
    granule_gain: f32;
    granule_octaves: f32;
    sunspot_sharpness: f32;
    sunspot_cutoff: f32;
    sunspot_frequency: f32;
};

[[group(1), binding(0)]]
var<uniform> star: Star;

// Signed distance function for a sphere of radius r
fn distance_from_sphere(p: vec3<f32>, c: vec3<f32>, r: f32) -> f32 {
    return length(p - c) - r;
}

fn distance_function(pos: vec3<f32>) -> f32 {
    return distance_from_sphere(pos, star.pos.xyz, star.pos.w);
}

fn calculate_normal(pos: vec3<f32>) -> vec3<f32> {

    let small_step = vec3<f32>(0.001, 0.0, 0.0);

    let gradient_x = distance_function(pos + small_step.xyy) - distance_function(pos - small_step.xyy);
    let gradient_y = distance_function(pos + small_step.yxy) - distance_function(pos - small_step.yxy);
    let gradient_z = distance_function(pos + small_step.yyx) - distance_function(pos - small_step.yyx);

    return normalize(vec3<f32>(gradient_x, gradient_y, gradient_z));
}


struct FragmentOutput {
    [[builtin(frag_depth)]] depth: f32;
    [[location(0)]] color: vec4<f32>;
};

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
    out.color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.depth = 1.0;

    // Ray marching

    var total_distance: f32 = 0.0;

    var i: i32 = 0;

    loop {
        let current_pos = origin + total_distance * direction;
        let distance_to_closest = distance_function(current_pos);

        if (distance_to_closest < MIN_HIT_DISTANCE) {
            // On hit
            // let world_pos = origin + distance_to_closest * direction;
            let clip_pos = env.world_to_clip * vec4<f32>(current_pos, 1.0);

            let surface_pos = normalize(current_pos - star.pos.xyz);

            let n = (noise(vec4<f32>(surface_pos * star.pos.w, env.time), star.granule_lacunarity, star.granule_gain, star.granule_octaves) + 1.0) * 0.5;

            let t1 = snoise(vec4<f32>(surface_pos * star.pos.w * star.sunspot_frequency, env.time)) - star.sunspot_cutoff;
            let t2 = snoise(vec4<f32>((surface_pos + 1.0) * star.pos.w * star.sunspot_frequency, env.time)) - star.sunspot_cutoff;
            let ss = (max(t1, 0.0) * max(t2, 0.0)) * star.sunspot_sharpness;
            let total = n - ss;

            let normal = calculate_normal(current_pos);

            let theta = 1.0 - dot(normal, -direction);

            out.color = vec4<f32>(total * star.color.xyz + theta * star.shift.xyz, 1.0);
            out.depth = clip_pos.z / clip_pos.w;
            break;
        } 

        if (total_distance > MAX_TRACE_DISTANCE || i > NUMBER_OF_STEPS) {
            // On Miss

            discard;
        }

        total_distance = total_distance + distance_to_closest;
        i = i + 1;
    }

    return out;
}

[[block]]
struct Settings {
    offset: vec2<u32>;
    size: vec2<u32>;
};


[[group(0), binding(0)]]
var color: texture_storage_2d<rgba16float, write>;

[[group(0), binding(1)]]
var depth: texture_2d<f32>;

[[block]]
struct Enviornment {
    proj_view: mat4x4<f32>;
    inv_proj_view: mat4x4<f32>;
    camera: vec4<f32>;
    time: f32;
};

[[group(0), binding(2)]]
var<uniform> env: Enviornment;


[[stage(compute), workgroup_size(4, 4, 1)]]
fn main([[builtin(global_invocation_id)]] invocation: vec3<u32>) {
    
}