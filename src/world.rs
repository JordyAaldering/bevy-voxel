use bevy::{prelude::*, platform::collections::HashMap};

use crate::{chunk::*, voxel::*};

use super::VoxelType;

#[derive(Default, Resource)]
pub struct VoxelWorld {
    pub chunks: HashMap<ChunkPosition, Chunk>,
}

impl VoxelWorld {
    pub fn is_solid(&self, pos: WorldPosition) -> bool {
        self.get_voxel(pos)
            .is_some_and(|voxel| !voxel.is_transparent())
    }

    pub fn get_voxel(&self, pos: WorldPosition) -> Option<VoxelType> {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return None;
        }

        let local_x = pos.x.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let local_y = pos.y as usize;
        let local_z = pos.z.rem_euclid(CHUNK_WIDTH as i32) as usize;

        let local_pos = VoxelPosition::new(local_x, local_y, local_z);

        let chunk = self.get_chunk(pos)?;
        Some(chunk[local_pos])
    }

    pub fn get_chunk(&self, pos: WorldPosition) -> Option<&Chunk> {
        let chunk_x = pos.x.div_euclid(CHUNK_WIDTH as i32);
        let chunk_z = pos.z.div_euclid(CHUNK_WIDTH as i32);
        let chunk_position = ChunkPosition::new(chunk_x, chunk_z);
        self.chunks.get(&chunk_position)
    }

    pub fn get_chunk_neighbor(&self, position: ChunkPosition, dx: i32, dz: i32) -> Option<&Chunk> {
        let neighbor_position = ChunkPosition::new(position.x + dx,position.z + dz);
        self.chunks.get(&neighbor_position)
    }
}
