struct VoronoiResult {
    // Distance to the nearest feature point (squared for `voronoi_3d`, to the nearest border for `voronoi_precise_3d`)
    value: f32,
    // Random value of the nearest cell
    cell: f32,
}

fn voronoi_random_vector(uv: vec3<f32>, offset: f32) -> vec3<f32> {
    // WGSL matrices are column-major, so this has the same columns as the HLSL rows,
    // and `m * uv` equals the HLSL `mul(uv, m)`.
    let m = mat3x3<f32>(
        15.27, 47.63, 99.41,
        89.98, 95.07, 38.39,
        33.83, 51.06, 60.77,
    );
    let hashed = fract(sin(m * uv) * 46839.32);
    return vec3<f32>(
        sin(hashed.y * offset) * 0.5 + 0.5,
        cos(hashed.x * offset) * 0.5 + 0.5,
        sin(hashed.z * offset) * 0.5 + 0.5,
    );
}

// Distance to the cell borders, instead of to the feature points, by also checking the cells around the nearest one.
fn voronoi_precise_3d(uv: vec3<f32>, angle_offset: f32, cell_density: f32) -> VoronoiResult {
    let g = floor(uv * cell_density);
    let f = fract(uv * cell_density);
    var res = vec2<f32>(8.0, 8.0);
    var ml = vec3<f32>(0.0);
    var mv = vec3<f32>(0.0);

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            for (var z = -1; z <= 1; z++) {
                let lattice = vec3<f32>(f32(x), f32(y), f32(z));
                let offset = voronoi_random_vector(g + lattice, angle_offset);
                let v = lattice + offset - f;
                let d = dot(v, v);

                if (d < res.x) {
                    res.x = d;
                    res.y = offset.x;
                    mv = v;
                    ml = lattice;
                }
            }
        }
    }

    let cell = res.y;

    res = vec2<f32>(8.0, 8.0);
    for (var y = -2; y <= 2; y++) {
        for (var x = -2; x <= 2; x++) {
            for (var z = -2; z <= 2; z++) {
                let lattice = ml + vec3<f32>(f32(x), f32(y), f32(z));
                let offset = voronoi_random_vector(g + lattice, angle_offset);
                let v = lattice + offset - f;

                let cell_difference = abs(ml - lattice);
                if (cell_difference.x + cell_difference.y + cell_difference.z > 0.1) {
                    let d = dot(0.5 * (mv + v), normalize(v - mv));
                    res.x = min(res.x, d);
                }
            }
        }
    }

    return VoronoiResult(res.x, cell);
}

fn voronoi_3d(uv: vec3<f32>, angle_offset: f32, cell_density: f32) -> VoronoiResult {
    let g = floor(uv * cell_density);
    let f = fract(uv * cell_density);
    var res = vec3<f32>(8.0, 8.0, 8.0);

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            for (var z = -1; z <= 1; z++) {
                let lattice = vec3<f32>(f32(x), f32(y), f32(z));
                let offset = voronoi_random_vector(g + lattice, angle_offset);
                let v = lattice + offset - f;
                let d = dot(v, v);

                if (d < res.x) {
                    res.y = res.x;
                    res.x = d;
                    res.z = offset.x;
                } else if (d < res.y) {
                    res.y = d;
                }
            }
        }
    }

    return VoronoiResult(res.x, res.z);
}
