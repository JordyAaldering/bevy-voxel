// https://github.com/KdotJPG/OpenSimplex2/blob/master/hlsl/OpenSimplex2S.hlsl
fn permute(t: vec4<f32>) -> vec4<f32> {
    return t * (t * 34.0 + 133.0);
}

fn mod1(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

fn mod3(x: vec3<f32>, y: vec3<f32>) -> vec3<f32> {
    return x - y * floor(x / y);
}

fn mod4(x: vec4<f32>, y: vec4<f32>) -> vec4<f32> {
    return x - y * floor(x / y);
}

// Gradient set is a normalized expanded rhombic dodecahedron
fn gradient(hash: f32) -> vec3<f32> {
    // Random vertex of a cube, +/- 1 each
    let cube = mod3(floor(hash / vec3<f32>(1.0, 2.0, 4.0)), vec3<f32>(2.0)) * 2.0 - 1.0;

    // Random edge of the three edges connected to that vertex
    var cuboct = cube;
    let index = i32(hash / 16.0);
    if (index == 0) {
        cuboct.x = 0.0;
    } else if (index == 1) {
        cuboct.y = 0.0;
    } else {
        cuboct.z = 0.0;
    }

    // In a funky way, pick one of the four points on the rhombic face
    let kind = mod1(floor(hash / 8.0), 2.0);
    let rhomb = (1.0 - kind) * cube + kind * (cuboct + cross(cube, cuboct));

    // Expand it so that the new edges are the same length as the existing ones
    var g = cuboct * 1.22474487139 + rhomb;

    // Shorten the second type of vector so all gradients have the same length,
    // and apply the noise scale constant.
    g *= (1.0 - 0.042942436724648037 * kind) * 3.5946317686139184;

    return g;
}

// BCC lattice split up into 2 cube lattices
fn open_simplex_2s_derivatives_part(X: vec3<f32>) -> vec4<f32> {
    let b = floor(X);
    let i4 = vec4<f32>(X - b, 2.5);

    // Pick between each pair of opposite corners in the cube.
    let v1 = b + floor(dot(i4, vec4<f32>(0.25, 0.25, 0.25, 0.25)));
    let v2 = b + vec3<f32>(1.0, 0.0, 0.0) + vec3<f32>(-1.0, 1.0, 1.0) * floor(dot(i4, vec4<f32>(-0.25, 0.25, 0.25, 0.35)));
    let v3 = b + vec3<f32>(0.0, 1.0, 0.0) + vec3<f32>(1.0, -1.0, 1.0) * floor(dot(i4, vec4<f32>(0.25, -0.25, 0.25, 0.35)));
    let v4 = b + vec3<f32>(0.0, 0.0, 1.0) + vec3<f32>(1.0, 1.0, -1.0) * floor(dot(i4, vec4<f32>(0.25, 0.25, -0.25, 0.35)));

    // Gradient hashes for the four vertices in this half-lattice.
    let m289 = vec4<f32>(289.0);
    var hashes = permute(mod4(vec4<f32>(v1.x, v2.x, v3.x, v4.x), m289));
    hashes = permute(mod4(hashes + vec4<f32>(v1.y, v2.y, v3.y, v4.y), m289));
    hashes = mod4(permute(mod4(hashes + vec4<f32>(v1.z, v2.z, v3.z, v4.z), m289)), vec4<f32>(48.0));

    // Gradient extrapolations & kernel function
    let d1 = X - v1;
    let d2 = X - v2;
    let d3 = X - v3;
    let d4 = X - v4;
    let a = max(0.75 - vec4<f32>(dot(d1, d1), dot(d2, d2), dot(d3, d3), dot(d4, d4)), vec4<f32>(0.0));
    let aa = a * a;
    let aaaa = aa * aa;
    let g1 = gradient(hashes.x);
    let g2 = gradient(hashes.y);
    let g3 = gradient(hashes.z);
    let g4 = gradient(hashes.w);
    let extrapolations = vec4<f32>(dot(d1, g1), dot(d2, g2), dot(d3, g3), dot(d4, g4));

    // Derivatives of the noise
    let w = aa * a * extrapolations;
    let derivative = -8.0 * (w.x * d1 + w.y * d2 + w.z * d3 + w.w * d4)
        + (aaaa.x * g1 + aaaa.y * g2 + aaaa.z * g3 + aaaa.w * g4);

    return vec4<f32>(derivative, dot(aaaa, extrapolations));
}

// Use this if you don't want Z to look different from X and Y
fn open_simplex_2s_derivatives_conventional(p: vec3<f32>) -> vec4<f32> {
    let X = dot(p, vec3<f32>(2.0 / 3.0)) - p;

    let result = open_simplex_2s_derivatives_part(X) + open_simplex_2s_derivatives_part(X + vec3<f32>(144.5));

    return vec4<f32>(dot(result.xyz, vec3<f32>(2.0 / 3.0)) - result.xyz, result.w);
}

// Use this if you want to show X and Y in a plane, then use Z for time, vertical, etc.
fn open_simplex_2s_derivatives_improve_xy(p: vec3<f32>) -> vec4<f32> {
    // Not a skew transform. WGSL matrices are column-major, so the columns
    // here are the rows of the original HLSL matrix.
    let orthonormal_map = mat3x3<f32>(
        vec3<f32>(0.788675134594813, -0.211324865405187, -0.577350269189626),
        vec3<f32>(-0.211324865405187, 0.788675134594813, -0.577350269189626),
        vec3<f32>(0.577350269189626, 0.577350269189626, 0.577350269189626),
    );

    let X = orthonormal_map * p;
    let result = open_simplex_2s_derivatives_part(X) + open_simplex_2s_derivatives_part(X + vec3<f32>(144.5));

    return vec4<f32>(result.xyz * orthonormal_map, result.w);
}
