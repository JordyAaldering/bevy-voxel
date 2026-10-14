use bevy::math::{Vec2, Vec3};

use crate::{chunk::*, face::{Face, FaceDirection}, voxel::VoxelPosition, world::VoxelWorld};

use super::VoxelType;

#[derive(Default)]
pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
}

impl MeshData {

}
