use std::ops::{Index, IndexMut};

use bevy::math::{Vec3, vec3};

use crate::{face::FaceDirection, voxel::VoxelPosition};

use super::VoxelType;

pub const CHUNK_WIDTH: usize = 3;
pub const CHUNK_HEIGHT: usize = 4;

pub struct Chunk {
    voxels: [VoxelType; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl Default for Chunk {
    fn default() -> Self {
        Self { voxels: [VoxelType::Air; _] }
    }
}

impl Chunk {
    pub fn get_neighbor_voxel(&self, pos: VoxelPosition, direction: FaceDirection) -> Option<VoxelType> {
        let (x, y, z) = (pos.x, pos.y, pos.z);
        let (nx, ny, nz) = match direction {
            FaceDirection::Right => (x + 1, y, z),
            FaceDirection::Left => (x.wrapping_sub(1), y, z),
            FaceDirection::Top => (x, y + 1, z),
            FaceDirection::Bottom => (x, y.wrapping_sub(1), z),
            FaceDirection::Front => (x, y, z + 1),
            FaceDirection::Back => (x, y, z.wrapping_sub(1)),
        };

        if nx < CHUNK_WIDTH && ny < CHUNK_HEIGHT && nz < CHUNK_WIDTH {
            let pos = VoxelPosition { x: nx, y: ny, z: nz };
            let idx = pos.to_index();
            Some(self.voxels[idx])
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (VoxelPosition, VoxelType)> {
        (0..CHUNK_WIDTH).flat_map(move |z| {
            (0..CHUNK_HEIGHT).flat_map(move |y| {
                (0..CHUNK_WIDTH).map(move |x| {
                    let pos = VoxelPosition { x, y, z };
                    let voxel = self.voxels[pos.to_index()];
                    (pos, voxel)
                })
            })
        })
    }
}

impl Index<VoxelPosition> for Chunk {
    type Output = VoxelType;

    fn index(&self, pos: VoxelPosition) -> &Self::Output {
        &self.voxels[pos.to_index()]
    }
}

impl IndexMut<VoxelPosition> for Chunk {
    fn index_mut(&mut self, pos: VoxelPosition) -> &mut Self::Output {
        &mut self.voxels[pos.to_index()]
    }
}

impl ChunkPosition {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// This chunk's origin in world space.
    pub fn origin(self) -> Vec3 {
        vec3(
            self.x as f32 * CHUNK_WIDTH as f32,
            0.0,
            self.z as f32 * CHUNK_WIDTH as f32,
        )
    }
}
