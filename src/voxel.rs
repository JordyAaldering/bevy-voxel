use bevy::math::{Vec3, vec3};

use crate::{chunk::{CHUNK_HEIGHT, CHUNK_WIDTH}, face::FaceDirection, texture_atlas::TextureAtlasPosition};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoxelType {
    Air,
    Grass,
    Dirt,
    Stone,
}

impl VoxelType {
    pub fn is_transparent(&self) -> bool {
        match self {
            Self::Air => true,
            _ => false,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self {
            Self::Air => false,
            _ => true,
        }
    }

    pub fn texture(self, face: FaceDirection) -> TextureAtlasPosition {
        match self {
            Self::Air => unreachable!(),
            Self::Grass => {
                match face {
                    FaceDirection::Top => TextureAtlasPosition::new(3, 1),
                    FaceDirection::Bottom => TextureAtlasPosition::new(1, 0),
                    _ => TextureAtlasPosition::new(2, 0),
                }
            },
            Self::Dirt => TextureAtlasPosition::new(1, 0),
            Self::Stone => TextureAtlasPosition::new(0, 0),
        }
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
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < CHUNK_WIDTH);
        Self { x, y, z }
    }

    pub fn origin(&self) -> Vec3 {
        vec3(self.x as f32, self.y as f32, self.z as f32)
    }

    pub fn to_index(self) -> usize {
        self.x + CHUNK_WIDTH * (self.y + CHUNK_HEIGHT * self.z)
    }
}

/// A voxel's position in world space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn origin(&self) -> Vec3 {
        vec3(self.x as f32, self.y as f32, self.z as f32)
    }

    pub fn neighbor(&self, face: FaceDirection) -> Self {
        match face {
            FaceDirection::Left   => Self::new(self.x - 1, self.y, self.z),
            FaceDirection::Right  => Self::new(self.x + 1, self.y, self.z),
            FaceDirection::Top    => Self::new(self.x, self.y + 1, self.z),
            FaceDirection::Bottom => Self::new(self.x, self.y - 1, self.z),
            FaceDirection::Front  => Self::new(self.x, self.y, self.z + 1),
            FaceDirection::Back   => Self::new(self.x, self.y, self.z - 1),
        }
    }
}
