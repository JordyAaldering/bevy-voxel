use std::ops::{Index, IndexMut};

use bevy::math::{Vec3, vec3};

use crate::face::FaceDirection;

use super::VoxelType;

pub const CHUNK_WIDTH: usize = 3;
pub const CHUNK_HEIGHT: usize = 4;

pub struct Chunk {
    voxels: [VoxelType; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl ChunkPosition {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn world_position(self) -> Vec3 {
        vec3(
            self.x as f32 * CHUNK_WIDTH as f32,
            0.0,
            self.z as f32 * CHUNK_WIDTH as f32,
        )
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self { voxels: [VoxelType::Air; _] }
    }
}

impl Chunk {
    pub fn xyz_to_index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < CHUNK_WIDTH);
        x + CHUNK_WIDTH * (y + CHUNK_HEIGHT * z)
    }

    pub fn try_get_neighbor_voxel(&self, x: usize, y: usize, z: usize, direction: FaceDirection) -> Option<VoxelType> {
        let (nx, ny, nz) = match direction {
            FaceDirection::Right => (x + 1, y, z),
            FaceDirection::Left => (x.wrapping_sub(1), y, z),
            FaceDirection::Top => (x, y + 1, z),
            FaceDirection::Bottom => (x, y.wrapping_sub(1), z),
            FaceDirection::Front => (x, y, z + 1),
            FaceDirection::Back => (x, y, z.wrapping_sub(1)),
        };

        if nx < CHUNK_WIDTH && ny < CHUNK_HEIGHT && nz < CHUNK_WIDTH {
            let idx = Self::xyz_to_index(nx, ny, nz);
            Some(self.voxels[idx])
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, usize, VoxelType)> {
        (0..CHUNK_WIDTH).flat_map(move |z| {
            (0..CHUNK_HEIGHT).flat_map(move |y| {
                (0..CHUNK_WIDTH).map(move |x| {
                    let idx = Self::xyz_to_index(x, y, z);
                    let voxel = self.voxels[idx];
                    (x, y, z, voxel)
                })
            })
        })
    }
}

impl Index<(usize, usize, usize)> for Chunk {
    type Output = VoxelType;

    fn index(&self, (x, y, z): (usize, usize, usize)) -> &Self::Output {
        let idx = Self::xyz_to_index(x, y, z);
        &self.voxels[idx]
    }
}

impl IndexMut<(usize, usize, usize)> for Chunk {
    fn index_mut(&mut self, (x, y, z): (usize, usize, usize)) -> &mut Self::Output {
        let idx = Self::xyz_to_index(x, y, z);
        &mut self.voxels[idx]
    }
}
