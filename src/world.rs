use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};

use crate::{chunk::*, voxel::*};

use super::VoxelType;

#[derive(Default, Resource)]
pub struct VoxelWorld {
    pub chunks: HashMap<ChunkPosition, Chunk>,
    dirty_chunks: HashSet<ChunkPosition>,
}

impl VoxelWorld {
    pub fn is_solid(&self, pos: WorldPosition) -> bool {
        self.get_voxel(pos)
            .is_some_and(|voxel| voxel.is_solid())
    }

    pub fn get_voxel(&self, pos: WorldPosition) -> Option<VoxelType> {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return None;
        }

        let voxel_pos = world_to_voxel_position(pos);
        let chunk_pos = world_to_chunk_position(pos);
        self.chunks.get(&chunk_pos).map(|chunk| chunk[voxel_pos])
    }

    pub fn set_voxel(&mut self, pos: WorldPosition, voxel: VoxelType) {
        let voxel_pos = world_to_voxel_position(pos);
        let chunk_pos = world_to_chunk_position(pos);
        let chunk = self.chunks.get_mut(&chunk_pos).unwrap();
        self.dirty_chunks.insert(chunk_pos);
        chunk[voxel_pos] = voxel;
    }

    pub fn get_chunk_neighbor(&self, position: ChunkPosition, dx: i32, dz: i32) -> Option<&Chunk> {
        let neighbor_position = ChunkPosition::new(position.x + dx,position.z + dz);
        self.chunks.get(&neighbor_position)
    }
}

fn world_to_chunk_position(pos: WorldPosition) -> ChunkPosition {
    let chunk_x = pos.x.div_euclid(CHUNK_WIDTH as i32);
    let chunk_z = pos.z.div_euclid(CHUNK_WIDTH as i32);
    ChunkPosition::new(chunk_x, chunk_z)
}

fn world_to_voxel_position(pos: WorldPosition) -> VoxelPosition {
    let local_x = pos.x.rem_euclid(CHUNK_WIDTH as i32) as usize;
    let local_y = pos.y as usize;
    let local_z = pos.z.rem_euclid(CHUNK_WIDTH as i32) as usize;
    VoxelPosition::new(local_x, local_y, local_z)
}
