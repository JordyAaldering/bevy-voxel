use bevy::math::{Vec3, vec3};

/// A voxel's position in world space.
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

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
