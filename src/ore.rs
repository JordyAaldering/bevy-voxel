use crate::{VoxelType, noise::voronoi};

use serde::{Deserialize, Serialize};

/// Voronoi cells are this many times wider than the largest cluster radius, so clusters
/// stay well apart and never get cut off by the edge of their cell.
const CELL_SIZE_FACTOR: f64 = 4.0;
/// The smallest a cluster gets, as a fraction of `Ore::cluster_size`.
const MIN_CLUSTER_FRACTION: f64 = 0.5;

const RADIUS_SALT: u64 = 0x5241_4449;
const RARITY_SALT: u64 = 0x5241_5249;

/// A type of ore, which spawns in clusters inside stone.
///
/// Clusters are placed with Voronoi noise: every cell has one feature point, which is the center
/// of a cluster if the cell is picked by `rarity`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ore {
    pub name: String,
    /// The voxel that replaces stone.
    pub voxel: VoxelType,
    /// Lowest world height (y) the ore is found at, inclusive.
    pub min_height: i32,
    /// Highest world height (y) the ore is found at, inclusive.
    pub max_height: i32,
    /// Probability that a Voronoi cell holds a cluster, between 0.0 and 1.0.
    /// A cell is about `4 * cluster_size` blocks wide, so smaller clusters need a lower rarity
    /// to give the same amount of ore in total.
    pub rarity: f32,
    /// Radius of the largest clusters, in blocks. Each cluster is between half this and the full size.
    pub cluster_size: f32,
}

impl Ore {
    /// Whether this ore occupies the voxel at the given world position.
    /// A pure function of the position and seed, so chunks generate independently.
    /// `seed` should differ per ore, so that clusters of different ores do not coincide.
    pub fn contains(&self, seed: i64, x: i32, y: i32, z: i32) -> bool {
        if y < self.min_height || y > self.max_height || self.cluster_size <= 0.0 {
            return false;
        }

        let cell_size = self.cluster_size as f64 * CELL_SIZE_FACTOR;
        let cell = voronoi::cell3(seed, x as f64 / cell_size, y as f64 / cell_size, z as f64 / cell_size);

        if unit(cell.hash, RARITY_SALT) >= self.rarity as f64 {
            return false;
        }

        let fraction = MIN_CLUSTER_FRACTION + (1.0 - MIN_CLUSTER_FRACTION) * unit(cell.hash, RADIUS_SALT);
        cell.distance * cell_size <= self.cluster_size as f64 * fraction
    }
}

/// Value in `[0, 1)` derived from a cell hash. The salt makes properties of one cell independent.
fn unit(hash: i64, salt: u64) -> f64 {
    let mut h = hash as u64 ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    (h >> 40) as f64 / (1u64 << 24) as f64
}
