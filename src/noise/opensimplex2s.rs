//!
//! K.jpg's OpenSimplex 2, smooth variant ("SuperSimplex")
//!

use std::{num::Wrapping, sync::{LazyLock, Once}};

const PRIME_X: i64 = 0x5205402B9270C86F;
const PRIME_Y: i64 = 0x598CD327003817B5;
const PRIME_Z: i64 = 0x5BCC226E9FA0BACB;
const PRIME_W: i64 = 0x56CC5227E58F554B;
const HASH_MULTIPLIER: i64 = 0x53A3F72DEEC546F5;
const SEED_FLIP_3D: i64 = -0x52D547B2E96ED629;

const ROOT2OVER2: f64 = 0.7071067811865476;
const SKEW_2D: f64 = 0.366025403784439;
const UNSKEW_2D: f64 = -0.21132486540518713;

const ROOT3OVER3: f64 = 0.577350269189626;
const FALLBACK_ROTATE3: f64 = 2.0 / 3.0;
const ROTATE3_ORTHOGONALIZER: f64 = UNSKEW_2D;

const SKEW_4D: f32 = 0.309016994374947;
const UNSKEW_4D: f32 = -0.138196601125011;

const N_GRADS_2D_EXPONENT: i32 = 7;
const N_GRADS_3D_EXPONENT: i32 = 8;
const N_GRADS_4D_EXPONENT: i32 = 9;
const N_GRADS_2D: i32 = 1 << N_GRADS_2D_EXPONENT;
const N_GRADS_3D: i32 = 1 << N_GRADS_3D_EXPONENT;
const N_GRADS_4D: i32 = 1 << N_GRADS_4D_EXPONENT;

const NORMALIZER_2D: f64 = 0.05481866495625118;
const NORMALIZER_3D: f64 = 0.2781926117527186;
const NORMALIZER_4D: f64 = 0.11127401889945551;

const RSQUARED_2D: f32 = 2.0 / 3.0;
const RSQUARED_3D: f32 = 3.0 / 4.0;
const RSQUARED_4D: f32 = 4.0 / 5.0;

/*
    Noise Evaluators
*/

/**
    2D OpenSimplex2S/SuperSimplex noise, standard lattice orientation.
*/
pub fn noise2(seed: i64, x: f64, y: f64) -> f32 {
    // Get points for A2* lattice
    let s = SKEW_2D * (x + y);
    let xs = x + s;
    let ys = y + s;

    noise2_UnskewedBase(seed, xs, ys)
}

/**
    2D  OpenSimplex2S/SuperSimplex noise base.
*/
fn noise2_UnskewedBase(seed: i64, xs: f64, ys: f64) -> f32 {
    let seed = Wrapping(seed);

    // Get base points and offsets.
    let xsb = fastFloor(xs);
    let ysb = fastFloor(ys);
    let xi = (xs - xsb as f64) as f32;
    let yi = (ys - ysb as f64) as f32;

    // Prime pre-multiplication for hash.
    let xsbp = Wrapping(xsb as i64) * Wrapping(PRIME_X);
    let ysbp = Wrapping(ysb as i64) * Wrapping(PRIME_Y);

    // Unskew.
    let t = (xi + yi) * UNSKEW_2D as f32;
    let dx0 = xi + t;
    let dy0 = yi + t;

    // First vertex.
    let a0 = RSQUARED_2D - dx0 * dx0 - dy0 * dy0;
    let mut value = (a0 * a0) * (a0 * a0) * grad2(seed, xsbp, ysbp, dx0, dy0);

    // Second vertex.
    let a1 = (2.0 * (1.0 + 2.0 * UNSKEW_2D) * (1.0 / UNSKEW_2D + 2.0)) as f32 * t
        + ((-2.0 * (1.0 + 2.0 * UNSKEW_2D) * (1.0 + 2.0 * UNSKEW_2D)) as f32 + a0);
    let dx1 = dx0 - (1.0 + 2.0 * UNSKEW_2D) as f32;
    let dy1 = dy0 - (1.0 + 2.0 * UNSKEW_2D) as f32;
    value += (a1 * a1)
        * (a1 * a1)
        * grad2(
            seed,
            xsbp + Wrapping(PRIME_X),
            ysbp + Wrapping(PRIME_Y),
            dx1,
            dy1,
        );

    // Third and fourth vertices.
    // Nested conditionals were faster than compact bit logic/arithmetic.
    let xmyi = xi - yi;
    if t < UNSKEW_2D as f32 {
        if xi + xmyi > 1.0 {
            let dx2 = dx0 - (3.0 * UNSKEW_2D + 2.0) as f32;
            let dy2 = dy0 - (3.0 * UNSKEW_2D + 1.0) as f32;
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value += (a2 * a2)
                    * (a2 * a2)
                    * grad2(
                        seed,
                        xsbp + Wrapping(PRIME_X << 1),
                        ysbp + Wrapping(PRIME_Y),
                        dx2,
                        dy2,
                    );
            }
        } else {
            let dx2 = dx0 - UNSKEW_2D as f32;
            let dy2 = dy0 - (UNSKEW_2D + 1.0) as f32;
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value +=
                    (a2 * a2) * (a2 * a2) * grad2(seed, xsbp, ysbp + Wrapping(PRIME_Y), dx2, dy2);
            }
        }

        if yi - xmyi > 1.0 {
            let dx3 = dx0 - (3.0 * UNSKEW_2D + 1.0) as f32;
            let dy3 = dy0 - (3.0 * UNSKEW_2D + 2.0) as f32;
            let a3 = RSQUARED_2D - dx3 * dx3 - dy3 * dy3;
            if a3 > 0.0 {
                value += (a3 * a3)
                    * (a3 * a3)
                    * grad2(
                        seed,
                        xsbp + Wrapping(PRIME_X),
                        ysbp + Wrapping(PRIME_Y << 1),
                        dx3,
                        dy3,
                    );
            }
        } else {
            let dx3 = dx0 - (UNSKEW_2D + 1.0) as f32;
            let dy3 = dy0 - UNSKEW_2D as f32;
            let a3 = RSQUARED_2D - dx3 * dx3 - dy3 * dy3;
            if a3 > 0.0 {
                value +=
                    (a3 * a3) * (a3 * a3) * grad2(seed, xsbp + Wrapping(PRIME_X), ysbp, dx3, dy3);
            }
        }
    } else {
        if xi + xmyi < 0.0 {
            let dx2 = dx0 + (1.0 + UNSKEW_2D) as f32;
            let dy2 = dy0 + UNSKEW_2D as f32;
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value +=
                    (a2 * a2) * (a2 * a2) * grad2(seed, xsbp - Wrapping(PRIME_X), ysbp, dx2, dy2);
            }
        } else {
            let dx2 = dx0 - (UNSKEW_2D + 1.0) as f32;
            let dy2 = dy0 - UNSKEW_2D as f32;
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value +=
                    (a2 * a2) * (a2 * a2) * grad2(seed, xsbp + Wrapping(PRIME_X), ysbp, dx2, dy2);
            }
        }

        if yi < xmyi {
            let dx2 = dx0 + UNSKEW_2D as f32;
            let dy2 = dy0 + (UNSKEW_2D + 1.0) as f32;
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value +=
                    (a2 * a2) * (a2 * a2) * grad2(seed, xsbp, ysbp - Wrapping(PRIME_Y), dx2, dy2);
            }
        } else {
            let dx2 = dx0 - UNSKEW_2D as f32;
            let dy2 = dy0 - (UNSKEW_2D + 1.0) as f32;
            let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
            if a2 > 0.0 {
                value +=
                    (a2 * a2) * (a2 * a2) * grad2(seed, xsbp, ysbp + Wrapping(PRIME_Y), dx2, dy2);
            }
        }
    }

    value
}

/**
    3D OpenSimplex2S/SuperSimplex noise, with better visual isotropy in (X, Z).
    Recommended for 3D terrain and time-varied animations.
    The Y coordinate should always be the "different" coordinate in whatever your use case is.
    If Y is vertical in world coordinates, call noise3_ImproveXZ(x, Y, z).
    If Z is vertical in world coordinates, call noise3_ImproveXZ(x, Z, y) or use noise3_ImproveXY.
    For a time varied animation, call noise3_ImproveXZ(x, T, y) or use noise3_ImproveXY.
*/
pub fn noise3(seed: i64, x: f64, y: f64, z: f64) -> f32 {
    // Re-orient the cubic lattices without skewing, so Y points up the main lattice diagonal,
    // and the planes formed by XZ are moved far out of alignment with the cube faces.
    // Orthonormal rotation. Not a skew transform.
    let xz = x + z;
    let s2 = xz * -0.211324865405187;
    let yy = y * ROOT3OVER3;
    let xr = x + s2 + yy;
    let zr = z + s2 + yy;
    let yr = xz * -ROOT3OVER3 + yy;

    // Evaluate both lattices to form a BCC lattice.
    noise3_UnrotatedBase(seed, xr, yr, zr)
}

/**
    3D OpenSimplex2S/SuperSimplex noise, fallback rotation option
    Use noise3_ImproveXY or noise3_ImproveXZ instead, wherever appropriate.
    They have less diagonal bias. This function's best use is as a fallback.
*/
pub fn noise3_Fallback(seed: i64, x: f64, y: f64, z: f64) -> f32 {
    // Re-orient the cubic lattices via rotation, to produce a familiar look.
    // Orthonormal rotation. Not a skew transform.
    let r = FALLBACK_ROTATE3 * (x + y + z);
    let xr = r - x;
    let yr = r - y;
    let zr = r - z;

    // Evaluate both lattices to form a BCC lattice.
    noise3_UnrotatedBase(seed, xr, yr, zr)
}

/**
    Generate overlapping cubic lattices for 3D Re-oriented BCC noise.
    Lookup table implementation inspired by DigitalShadow.
    It was actually faster to narrow down the points in the loop itself,
    than to build up the index with enough info to isolate 8 points.
*/
fn noise3_UnrotatedBase(seed: i64, xr: f64, yr: f64, zr: f64) -> f32 {
    let seed = Wrapping(seed);

    // Get base points and offsets.
    let xrb = fastFloor(xr);
    let yrb = fastFloor(yr);
    let zrb = fastFloor(zr);
    let xi = (xr - xrb as f64) as f32;
    let yi = (yr - yrb as f64) as f32;
    let zi = (zr - zrb as f64) as f32;

    // Prime pre-multiplication for hash. Also flip seed for second lattice copy.
    let xrbp = Wrapping(xrb as i64) * Wrapping(PRIME_X);
    let yrbp = Wrapping(yrb as i64) * Wrapping(PRIME_Y);
    let zrbp = Wrapping(zrb as i64) * Wrapping(PRIME_Z);
    let seed2 = seed ^ Wrapping(SEED_FLIP_3D);

    // -1 if positive, 0 if negative.
    let xNMask = (-0.5 - xi) as i32;
    let yNMask = (-0.5 - yi) as i32;
    let zNMask = (-0.5 - zi) as i32;

    // First vertex.
    let x0 = xi + xNMask as f32;
    let y0 = yi + yNMask as f32;
    let z0 = zi + zNMask as f32;
    let a0 = RSQUARED_3D - x0 * x0 - y0 * y0 - z0 * z0;
    let mut value = (a0 * a0)
        * (a0 * a0)
        * grad3(
            seed,
            xrbp + (Wrapping(xNMask as i64) & Wrapping(PRIME_X)),
            yrbp + (Wrapping(yNMask as i64) & Wrapping(PRIME_Y)),
            zrbp + (Wrapping(zNMask as i64) & Wrapping(PRIME_Z)),
            x0,
            y0,
            z0,
        );

    // Second vertex.
    let x1 = xi - 0.5;
    let y1 = yi - 0.5;
    let z1 = zi - 0.5;
    let a1 = RSQUARED_3D - x1 * x1 - y1 * y1 - z1 * z1;
    value += (a1 * a1)
        * (a1 * a1)
        * grad3(
            seed2,
            xrbp + Wrapping(PRIME_X),
            yrbp + Wrapping(PRIME_Y),
            zrbp + Wrapping(PRIME_Z),
            x1,
            y1,
            z1,
        );

    // Shortcuts for building the remaining falloffs.
    // Derived by subtracting the polynomials with the offsets plugged in.
    let xAFlipMask0 = ((xNMask | 1) << 1) as f32 * x1;
    let yAFlipMask0 = ((yNMask | 1) << 1) as f32 * y1;
    let zAFlipMask0 = ((zNMask | 1) << 1) as f32 * z1;
    let xAFlipMask1 = (-2 - (xNMask << 2)) as f32 * x1 - 1.0;
    let yAFlipMask1 = (-2 - (yNMask << 2)) as f32 * y1 - 1.0;
    let zAFlipMask1 = (-2 - (zNMask << 2)) as f32 * z1 - 1.0;

    let mut skip5 = false;
    let a2 = xAFlipMask0 + a0;
    if a2 > 0.0 {
        let x2 = x0 - (xNMask | 1) as f32;
        let y2 = y0;
        let z2 = z0;
        value += (a2 * a2)
            * (a2 * a2)
            * grad3(
                seed,
                xrbp + (Wrapping(!xNMask as i64) & Wrapping(PRIME_X)),
                yrbp + (Wrapping(yNMask as i64) & Wrapping(PRIME_Y)),
                zrbp + (Wrapping(zNMask as i64) & Wrapping(PRIME_Z)),
                x2,
                y2,
                z2,
            );
    } else {
        let a3 = yAFlipMask0 + zAFlipMask0 + a0;
        if a3 > 0.0 {
            let x3 = x0;
            let y3 = y0 - (yNMask | 1) as f32;
            let z3 = z0 - (zNMask | 1) as f32;
            value += (a3 * a3)
                * (a3 * a3)
                * grad3(
                    seed,
                    xrbp + (Wrapping(xNMask as i64) & Wrapping(PRIME_X)),
                    yrbp + (Wrapping(!yNMask as i64) & Wrapping(PRIME_Y)),
                    zrbp + (Wrapping(!zNMask as i64) & Wrapping(PRIME_Z)),
                    x3,
                    y3,
                    z3,
                );
        }

        let a4 = xAFlipMask1 + a1;
        if a4 > 0.0 {
            let x4 = (xNMask | 1) as f32 + x1;
            let y4 = y1;
            let z4 = z1;
            value += (a4 * a4)
                * (a4 * a4)
                * grad3(
                    seed2,
                    xrbp + (Wrapping(xNMask as i64) & (Wrapping(PRIME_X) << 1)),
                    yrbp + Wrapping(PRIME_Y),
                    zrbp + Wrapping(PRIME_Z),
                    x4,
                    y4,
                    z4,
                );
            skip5 = true;
        }
    }

    let mut skip9 = false;
    let a6 = yAFlipMask0 + a0;
    if a6 > 0.0 {
        let x6 = x0;
        let y6 = y0 - (yNMask | 1) as f32;
        let z6 = z0;
        value += (a6 * a6)
            * (a6 * a6)
            * grad3(
                seed,
                xrbp + (Wrapping(xNMask as i64) & Wrapping(PRIME_X)),
                yrbp + (Wrapping(!yNMask as i64) & Wrapping(PRIME_Y)),
                zrbp + (Wrapping(zNMask as i64) & Wrapping(PRIME_Z)),
                x6,
                y6,
                z6,
            );
    } else {
        let a7 = xAFlipMask0 + zAFlipMask0 + a0;
        if a7 > 0.0 {
            let x7 = x0 - (xNMask | 1) as f32;
            let y7 = y0;
            let z7 = z0 - (zNMask | 1) as f32;
            value += (a7 * a7)
                * (a7 * a7)
                * grad3(
                    seed,
                    xrbp + (Wrapping(!xNMask as i64) & Wrapping(PRIME_X)),
                    yrbp + (Wrapping(yNMask as i64) & Wrapping(PRIME_Y)),
                    zrbp + (Wrapping(!zNMask as i64) & Wrapping(PRIME_Z)),
                    x7,
                    y7,
                    z7,
                );
        }

        let a8 = yAFlipMask1 + a1;
        if a8 > 0.0 {
            let x8 = x1;
            let y8 = (yNMask | 1) as f32 + y1;
            let z8 = z1;
            value += (a8 * a8)
                * (a8 * a8)
                * grad3(
                    seed2,
                    xrbp + Wrapping(PRIME_X),
                    yrbp + (Wrapping(yNMask as i64) & (Wrapping(PRIME_Y) << 1)),
                    zrbp + Wrapping(PRIME_Z),
                    x8,
                    y8,
                    z8,
                );
            skip9 = true;
        }
    }

    let mut skipD = false;
    let aA = zAFlipMask0 + a0;
    if aA > 0.0 {
        let xA = x0;
        let yA = y0;
        let zA = z0 - (zNMask | 1) as f32;
        value += (aA * aA)
            * (aA * aA)
            * grad3(
                seed,
                xrbp + (Wrapping(xNMask as i64) & Wrapping(PRIME_X)),
                yrbp + (Wrapping(yNMask as i64) & Wrapping(PRIME_Y)),
                zrbp + (Wrapping(!zNMask as i64) & Wrapping(PRIME_Z)),
                xA,
                yA,
                zA,
            );
    } else {
        let aB = xAFlipMask0 + yAFlipMask0 + a0;
        if aB > 0.0 {
            let xB = x0 - (xNMask | 1) as f32;
            let yB = y0 - (yNMask | 1) as f32;
            let zB = z0;
            value += (aB * aB)
                * (aB * aB)
                * grad3(
                    seed,
                    xrbp + (Wrapping(!xNMask as i64) & Wrapping(PRIME_X)),
                    yrbp + (Wrapping(!yNMask as i64) & Wrapping(PRIME_Y)),
                    zrbp + (Wrapping(zNMask as i64) & Wrapping(PRIME_Z)),
                    xB,
                    yB,
                    zB,
                );
        }

        let aC = zAFlipMask1 + a1;
        if aC > 0.0 {
            let xC = x1;
            let yC = y1;
            let zC = (zNMask | 1) as f32 + z1;
            value += (aC * aC)
                * (aC * aC)
                * grad3(
                    seed2,
                    xrbp + Wrapping(PRIME_X),
                    yrbp + Wrapping(PRIME_Y),
                    zrbp + (Wrapping(zNMask as i64) & (Wrapping(PRIME_Z) << 1)),
                    xC,
                    yC,
                    zC,
                );
            skipD = true;
        }
    }

    if !skip5 {
        let a5 = yAFlipMask1 + zAFlipMask1 + a1;
        if a5 > 0.0 {
            let x5 = x1;
            let y5 = (yNMask | 1) as f32 + y1;
            let z5 = (zNMask | 1) as f32 + z1;
            value += (a5 * a5)
                * (a5 * a5)
                * grad3(
                    seed2,
                    xrbp + Wrapping(PRIME_X),
                    yrbp + (Wrapping(yNMask as i64) & (Wrapping(PRIME_Y) << 1)),
                    zrbp + (Wrapping(zNMask as i64) & (Wrapping(PRIME_Z) << 1)),
                    x5,
                    y5,
                    z5,
                );
        }
    }

    if !skip9 {
        let a9 = xAFlipMask1 + zAFlipMask1 + a1;
        if a9 > 0.0 {
            let x9 = (xNMask | 1) as f32 + x1;
            let y9 = y1;
            let z9 = (zNMask | 1) as f32 + z1;
            value += (a9 * a9)
                * (a9 * a9)
                * grad3(
                    seed2,
                    xrbp + (Wrapping(xNMask as i64) & (Wrapping(PRIME_X) << 1)),
                    yrbp + Wrapping(PRIME_Y),
                    zrbp + (Wrapping(zNMask as i64) & (Wrapping(PRIME_Z) << 1)),
                    x9,
                    y9,
                    z9,
                );
        }
    }

    if !skipD {
        let aD = xAFlipMask1 + yAFlipMask1 + a1;
        if aD > 0.0 {
            let xD = (xNMask | 1) as f32 + x1;
            let yD = (yNMask | 1) as f32 + y1;
            let zD = z1;
            value += (aD * aD)
                * (aD * aD)
                * grad3(
                    seed2,
                    xrbp + (Wrapping(xNMask as i64) & (Wrapping(PRIME_X) << 1)),
                    yrbp + (Wrapping(yNMask as i64) & (Wrapping(PRIME_Y) << 1)),
                    zrbp + Wrapping(PRIME_Z),
                    xD,
                    yD,
                    zD,
                );
        }
    }

    value
}

/*
    Utility
*/

fn grad2(seed: Wrapping<i64>, xsvp: Wrapping<i64>, ysvp: Wrapping<i64>, dx: f32, dy: f32) -> f32 {
    let mut hash = seed ^ xsvp ^ ysvp;
    hash *= HASH_MULTIPLIER;
    hash ^= hash.0 >> (64 - N_GRADS_2D_EXPONENT + 1);
    let gi = (hash.0 as i32 & ((N_GRADS_2D - 1) << 1)) as usize;
    GRADIENTS_2D[gi | 0] * dx + GRADIENTS_2D[gi | 1] * dy
}

fn grad3(
    seed: Wrapping<i64>,
    xrvp: Wrapping<i64>,
    yrvp: Wrapping<i64>,
    zrvp: Wrapping<i64>,
    dx: f32,
    dy: f32,
    dz: f32,
) -> f32 {
    let mut hash = (seed ^ xrvp) ^ (yrvp ^ zrvp);
    hash *= HASH_MULTIPLIER;
    hash ^= hash.0 >> (64 - N_GRADS_3D_EXPONENT + 2);
    let gi = (hash.0 as i32 & ((N_GRADS_3D - 1) << 2)) as usize;
    GRADIENTS_3D[gi | 0] * dx + GRADIENTS_3D[gi | 1] * dy + GRADIENTS_3D[gi | 2] * dz
}

fn fastFloor(x: f64) -> i32 {
    let xi = x as i32;
    if x < xi as f64 {
        xi - 1
    } else {
        xi
    }
}

const GRAD2_SRC: &[f64] = &[
     0.382683432365090,  0.923879532511287,
     0.923879532511287,  0.382683432365090,
     0.923879532511287, -0.382683432365090,
     0.382683432365090, -0.923879532511287,
    -0.382683432365090, -0.923879532511287,
    -0.923879532511287, -0.382683432365090,
    -0.923879532511287,  0.382683432365090,
    -0.382683432365090,  0.923879532511287,
    //-----------------------------------//
     0.130526192220052,  0.991444861373810,
     0.608761429008721,  0.793353340291235,
     0.793353340291235,  0.608761429008721,
     0.991444861373810,  0.130526192220051,
     0.991444861373810, -0.130526192220051,
     0.793353340291235, -0.608761429008720,
     0.608761429008721, -0.793353340291235,
     0.130526192220052, -0.991444861373810,
    -0.130526192220052, -0.991444861373810,
    -0.608761429008721, -0.793353340291235,
    -0.793353340291235, -0.608761429008721,
    -0.991444861373810, -0.130526192220052,
    -0.991444861373810,  0.130526192220051,
    -0.793353340291235,  0.608761429008721,
    -0.608761429008721,  0.793353340291235,
    -0.130526192220052,  0.991444861373810,
];

const GRAD3_SRC: &[f64] = &[
     2.22474487139,       2.22474487139,      -1.0,                 0.0,
     2.22474487139,       2.22474487139,       1.0,                 0.0,
     3.0862664687972017,  1.1721513422464978,  0.0,                 0.0,
     1.1721513422464978,  3.0862664687972017,  0.0,                 0.0,
    -2.22474487139,       2.22474487139,      -1.0,                 0.0,
    -2.22474487139,       2.22474487139,       1.0,                 0.0,
    -1.1721513422464978,  3.0862664687972017,  0.0,                 0.0,
    -3.0862664687972017,  1.1721513422464978,  0.0,                 0.0,
    -1.0,                -2.22474487139,      -2.22474487139,       0.0,
     1.0,                -2.22474487139,      -2.22474487139,       0.0,
     0.0,                -3.0862664687972017, -1.1721513422464978,  0.0,
     0.0,                -1.1721513422464978, -3.0862664687972017,  0.0,
    -1.0,                -2.22474487139,       2.22474487139,       0.0,
     1.0,                -2.22474487139,       2.22474487139,       0.0,
     0.0,                -1.1721513422464978,  3.0862664687972017,  0.0,
     0.0,                -3.0862664687972017,  1.1721513422464978,  0.0,
    //----------------------------------------------------------------//
    -2.22474487139,      -2.22474487139,      -1.0,                 0.0,
    -2.22474487139,      -2.22474487139,       1.0,                 0.0,
    -3.0862664687972017, -1.1721513422464978,  0.0,                 0.0,
    -1.1721513422464978, -3.0862664687972017,  0.0,                 0.0,
    -2.22474487139,      -1.0,                -2.22474487139,       0.0,
    -2.22474487139,       1.0,                -2.22474487139,       0.0,
    -1.1721513422464978,  0.0,                -3.0862664687972017,  0.0,
    -3.0862664687972017,  0.0,                -1.1721513422464978,  0.0,
    -2.22474487139,      -1.0,                 2.22474487139,       0.0,
    -2.22474487139,       1.0,                 2.22474487139,       0.0,
    -3.0862664687972017,  0.0,                 1.1721513422464978,  0.0,
    -1.1721513422464978,  0.0,                 3.0862664687972017,  0.0,
    -1.0,                 2.22474487139,      -2.22474487139,       0.0,
     1.0,                 2.22474487139,      -2.22474487139,       0.0,
     0.0,                 1.1721513422464978, -3.0862664687972017,  0.0,
     0.0,                 3.0862664687972017, -1.1721513422464978,  0.0,
    -1.0,                 2.22474487139,       2.22474487139,       0.0,
     1.0,                 2.22474487139,       2.22474487139,       0.0,
     0.0,                 3.0862664687972017,  1.1721513422464978,  0.0,
     0.0,                 1.1721513422464978,  3.0862664687972017,  0.0,
     2.22474487139,      -2.22474487139,      -1.0,                 0.0,
     2.22474487139,      -2.22474487139,       1.0,                 0.0,
     1.1721513422464978, -3.0862664687972017,  0.0,                 0.0,
     3.0862664687972017, -1.1721513422464978,  0.0,                 0.0,
     2.22474487139,      -1.0,                -2.22474487139,       0.0,
     2.22474487139,       1.0,                -2.22474487139,       0.0,
     3.0862664687972017,  0.0,                -1.1721513422464978,  0.0,
     1.1721513422464978,  0.0,                -3.0862664687972017,  0.0,
     2.22474487139,      -1.0,                 2.22474487139,       0.0,
     2.22474487139,       1.0,                 2.22474487139,       0.0,
     1.1721513422464978,  0.0,                 3.0862664687972017,  0.0,
     3.0862664687972017,  0.0,                 1.1721513422464978,  0.0,
];

static GRADIENTS_2D: LazyLock<Vec<f32>> = LazyLock::new(|| {
    GRAD2_SRC
        .into_iter()
        .map(|v| (v / NORMALIZER_2D) as f32)
        .collect::<Vec<_>>() // cache divisions
        .into_iter()
        .cycle()
        .take((N_GRADS_2D * 2) as usize)
        .collect()
});

static GRADIENTS_3D: LazyLock<Vec<f32>> = LazyLock::new(|| {
    GRAD3_SRC
        .into_iter()
        .map(|v| (v / NORMALIZER_3D) as f32)
        .collect::<Vec<_>>() // cache divisions
        .into_iter()
        .cycle()
        .take((N_GRADS_3D * 4) as usize)
        .collect()
});
