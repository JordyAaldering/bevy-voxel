use bevy::{prelude::*, platform::collections::HashMap};

use crate::chunk::*;

use super::VoxelType;

#[derive(Default, Resource)]
pub struct VoxelWorld {
    pub chunks: HashMap<ChunkPosition, Chunk>,
}

impl VoxelWorld {
    pub fn get_voxel(&self, x: i32, y: i32, z: i32) -> Option<VoxelType> {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return None;
        }

        let chunk_x = x.div_euclid(CHUNK_WIDTH as i32);
        let chunk_z = z.div_euclid(CHUNK_WIDTH as i32);
        let local_x = x.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let local_y = y as usize;
        let local_z = z.rem_euclid(CHUNK_WIDTH as i32) as usize;

        let chunk_position = ChunkPosition::new(chunk_x, chunk_z);
        self.chunks.get(&chunk_position)
            .map(|chunk| chunk[(local_x, local_y, local_z)])
    }

    pub fn try_get_neighbor_chunk(&self, position: ChunkPosition, dx: i32, dz: i32) -> Option<&Chunk> {
        let neighbor_position = ChunkPosition::new(position.x + dx,position.z + dz);
        self.chunks.get(&neighbor_position)
    }
}
