#[allow(dead_code)]

use crate::{CHUNK_HEIGHT, CHUNK_WIDTH, direction::Direction};

use bevy::math::{Vec3, vec3};

/// Chunk-local voxel coordinate.
///
/// x ∈ [0, CHUNK_WIDTH)
/// y ∈ [0, CHUNK_HEIGHT)
/// z ∈ [0, CHUNK_WIDTH)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VoxelCoord {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl VoxelCoord {
    pub const fn new(x: usize, y: usize, z: usize) -> Self {
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < CHUNK_WIDTH);
        Self { x, y, z }
    }

    /// The flattened index of the voxel in the chunk's voxel 1D array.
    pub const fn index(self) -> usize {
        self.x + CHUNK_WIDTH * (self.y + CHUNK_HEIGHT * self.z)
    }

    /// Returns the local position of this voxel within its chunk.
    pub const fn local_position(self) -> Vec3 {
        vec3(self.x as f32, self.y as f32, self.z as f32)
    }
}

/// Coordinate of a voxel in global world space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldCoord {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns the local position of the voxel within its chunk.
    pub const fn local_coord(self) -> VoxelCoord {
        VoxelCoord {
            x: self.x.rem_euclid(CHUNK_WIDTH as i32) as usize,
            y: self.y as usize,
            z: self.z.rem_euclid(CHUNK_WIDTH as i32) as usize,
        }
    }

    /// Returns the coordinate of the chunk that contains this voxel.
    pub const fn chunk_coord(self) -> ChunkCoord {
        ChunkCoord {
            x: self.x.div_euclid(CHUNK_WIDTH as i32),
            z: self.z.div_euclid(CHUNK_WIDTH as i32),
        }
    }

    /// Returns the position of this voxel in world space.
    pub const fn position(self) -> Vec3 {
        vec3(self.x as f32, self.y as f32, self.z as f32)
    }

    pub const fn shift(self, direction: Direction) -> Self {
        let (dx, dy, dz) = direction.offset();
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }
}

/// Coordinate of a chunk in the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Returns the position of this chunk in world space.
    pub const fn position(self) -> Vec3 {
        vec3(self.x as f32 * CHUNK_WIDTH as f32, 0.0, self.z as f32 * CHUNK_WIDTH as f32)
    }
}
