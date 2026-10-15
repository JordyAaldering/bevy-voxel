use std::sync::LazyLock;

const PRIME_X: isize = 0x5205402B9270C86F;
const PRIME_Y: isize = 0x598CD327003817B5;
const PRIME_Z: isize = 0x5BCC226E9FA0BACB;
const HASH_MULTIPLIER: isize = 0x53A3F72DEEC546F5;

const SKEW_2D: f64 = 0.3660254037844386; // (sqrt(3) - 1.0) / 2
const UNSKEW_2D: f64 = 0.21132486540518713; // -(3 - sqrt(3)) / 6

const N_GRADS_2D_EXPONENT: usize = 7;
const N_GRADS_3D_EXPONENT: usize = 8;
const N_GRADS_2D: usize = 1 << N_GRADS_2D_EXPONENT;
const N_GRADS_3D: usize = 1 << N_GRADS_3D_EXPONENT;

const NORMALIZER_2D: f64 = 0.05481866495625118;
const NORMALIZER_3D: f64 = 0.2781926117527186;

const RSQUARED_2D: f64 = 2.0 / 3.0;
const RSQUARED_3D: f64 = 3.0 / 4.0;

/// https://github.com/KdotJPG/OpenSimplex2/blob/master/csharp/OpenSimplex2S.cs#L612
static GRADIENTS_2D: LazyLock<[f64; N_GRADS_2D * 2]> = LazyLock::new(|| {
    let mut gradients = [0.0; _];
    let mut grad2 = [
        0.38268343236509,   0.923879532511287,
        0.923879532511287,  0.38268343236509,
        0.923879532511287, -0.38268343236509,
        0.38268343236509,  -0.923879532511287,
        -0.38268343236509,  -0.923879532511287,
        -0.923879532511287, -0.38268343236509,
        -0.923879532511287,  0.38268343236509,
        -0.38268343236509,   0.923879532511287,
        //-------------------------------------//
        0.130526192220052,  0.99144486137381,
        0.608761429008721,  0.793353340291235,
        0.793353340291235,  0.608761429008721,
        0.99144486137381,   0.130526192220051,
        0.99144486137381,  -0.130526192220051,
        0.793353340291235, -0.60876142900872,
        0.608761429008721, -0.793353340291235,
        0.130526192220052, -0.99144486137381,
        -0.130526192220052, -0.99144486137381,
        -0.608761429008721, -0.793353340291235,
        -0.793353340291235, -0.608761429008721,
        -0.99144486137381,  -0.130526192220052,
        -0.99144486137381,   0.130526192220051,
        -0.793353340291235,  0.608761429008721,
        -0.608761429008721,  0.793353340291235,
        -0.130526192220052,  0.99144486137381,
    ];
    for i in 0..grad2.len() {
        grad2[i] /= NORMALIZER_2D;
    }
    for i in 0..gradients.len() {
        gradients[i] = grad2[i % grad2.len()];
    }
    gradients
});

/// https://github.com/KdotJPG/OpenSimplex2/blob/master/csharp/OpenSimplex2S.cs#L51
fn noise2(seed: isize, x: f64, y: f64) -> f64 {
    let s = SKEW_2D * (x + y);
    let xs = x + s;
    let ys = y + s;
    noise2_unskewed_base(seed, xs, ys)
}

/// https://github.com/KdotJPG/OpenSimplex2/blob/master/csharp/OpenSimplex2S.cs#L79
fn noise2_unskewed_base(seed: isize, xs: f64, ys: f64) -> f64 {
    // Get base points and offsets
    let xsb = xs.floor() as isize;
    let ysb = ys.floor() as isize;
    let xi = xs - xsb as f64;
    let yi = ys - ysb as f64;

    // Prime pre-multiplication for hash
    let xsbp = xsb * PRIME_X;
    let ysbp = ysb * PRIME_Y;

    // Unskew
    let t = (xi + yi) * UNSKEW_2D;
    let dx0 = xi + t;
    let dy0 = yi + t;

    // First vertex
    let a0 = RSQUARED_2D - dx0 * dx0 - dy0 * dy0;
    let mut value = (a0 * a0) * (a0 * a0) * grad(seed, xsbp, ysbp, dx0, dy0);

    // Second vertex
    let a1 = (2.0 * (2.0 * UNSKEW_2D + 1.0) * (1.0 / UNSKEW_2D + 2.0)) * t + ((-2.0 * (2.0 * UNSKEW_2D + 1.0) * (2.0 * UNSKEW_2D + 1.0)) + a0);
    let dx1 = dx0 - (2.0 * UNSKEW_2D + 1.0);
    let dy1 = dy0 - (2.0 * UNSKEW_2D + 1.0);
    value += (a1 * a1) * (a1 * a1) * grad(seed, xsbp + PRIME_X, ysbp + PRIME_Y, dx1, dy1);

    // Third and fourth vertices
    let xmyi = xi - yi;
    if t < UNSKEW_2D {
        if xi + xmyi > 1.0 {
            let dx2 = dx0 - (3.0 * UNSKEW_2D + 2.0);
            let dy2 = dy0 - (3.0 * UNSKEW_2D + 1.0);
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value += (a2 * a2) * (a2 * a2) * grad(seed, xsbp + (PRIME_X << 1), ysbp + PRIME_Y, dx2, dy2);
            }
        } else {
            let dx2 = dx0 - (UNSKEW_2D + 0.0);
            let dy2 = dy0 - (UNSKEW_2D + 1.0);
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value += (a2 * a2) * (a2 * a2) * grad(seed, xsbp, ysbp + PRIME_Y, dx2, dy2);
            }
        }

        if yi - xmyi > 1.0 {
            let dx3 = dx0 - (3.0 * UNSKEW_2D + 1.0);
            let dy3 = dy0 - (3.0 * UNSKEW_2D + 2.0);
            let a3 = RSQUARED_2D - dx3 * dx3 - dy3 * dy3;
            if a3 > 0.0 {
                value += (a3 * a3) * (a3 * a3) * grad(seed, xsbp + PRIME_X, ysbp + (PRIME_Y << 1), dx3, dy3);
            }
        } else {
            let dx3 = dx0 - (UNSKEW_2D + 1.0);
            let dy3 = dy0 - (UNSKEW_2D + 0.0);
            let a3 = RSQUARED_2D - dx3 * dx3 - dy3 * dy3;
            if a3 > 0.0 {
                value += (a3 * a3) * (a3 * a3) * grad(seed, xsbp + PRIME_X, ysbp, dx3, dy3);
            }
        }
    } else {
        if xi + xmyi < 0.0 {
            let dx2 = dx0 + (UNSKEW_2D + 1.0);
            let dy2 = dy0 + (UNSKEW_2D + 0.0);
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value += (a2 * a2) * (a2 * a2) * grad(seed, xsbp - PRIME_X, ysbp, dx2, dy2);
            }
        } else {
            let dx2 = dx0 - (UNSKEW_2D + 1.0);
            let dy2 = dy0 - (UNSKEW_2D + 0.0);
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value += (a2 * a2) * (a2 * a2) * grad(seed, xsbp + PRIME_X, ysbp, dx2, dy2);
            }
        }

        if yi < xmyi {
            let dx2 = dx0 + (UNSKEW_2D + 0.0);
            let dy2 = dy0 + (UNSKEW_2D + 1.0);
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value += (a2 * a2) * (a2 * a2) * grad(seed, xsbp, ysbp - PRIME_Y, dx2, dy2);
            }
        } else {
            let dx2 = dx0 - (UNSKEW_2D + 0.0);
            let dy2 = dy0 - (UNSKEW_2D + 1.0);
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value += (a2 * a2) * (a2 * a2) * grad(seed, xsbp, ysbp + PRIME_Y, dx2, dy2);
            }
        }
    }

    value
}

/// https://github.com/KdotJPG/OpenSimplex2/blob/master/csharp/OpenSimplex2S.cs#L563
fn grad(seed: isize, xsvp: isize, ysvp: isize, dx: f64, dy: f64) -> f64 {
    let mut hash = seed ^ xsvp ^ ysvp;
    hash *= HASH_MULTIPLIER;
    hash ^= hash >> (64 - N_GRADS_2D_EXPONENT + 1);
    let gi = (hash as usize) & ((N_GRADS_2D - 1) << 1);
    GRADIENTS_2D[gi | 0] * dx + GRADIENTS_2D[gi | 1] * dy
}
