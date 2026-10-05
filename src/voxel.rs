use bevy::math::{Vec3, vec3};

/// Chunk-local voxel position, with x and z in [0, CHUNK_WIDTH) and y in [0, CHUNK_HEIGHT).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VoxelPosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl VoxelPosition {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn origin(&self) -> Vec3 {
        vec3(self.x as f32, self.y as f32, self.z as f32)
    }
}
