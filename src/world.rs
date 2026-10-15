use bevy::{platform::collections::HashMap, prelude::*};

use crate::{chunk::*, opensimplex2s, voxel::*};

use super::VoxelType;

#[derive(Default, Resource)]
pub struct VoxelWorld {
    pub chunks: HashMap<ChunkPosition, ChunkData>,
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

    pub fn spawn_chunk(&mut self, commands: &mut Commands, position: ChunkPosition) -> Entity {
        let entity = commands.spawn((
            Chunk::new(position),
            Transform::from_translation(position.origin()),
            NeedsRemesh,
        )).id();

        let mut chunk = ChunkData::new(position, entity);

        for x in 0..CHUNK_WIDTH {
            let world_x = position.x * CHUNK_WIDTH as i32 + x as i32;
            for z in 0..CHUNK_WIDTH {
                let world_z = position.z * CHUNK_WIDTH as i32 + z as i32;

                let height = (opensimplex2s::noise2d(0, world_x as f64 / 100.0, world_z as f64 / 100.0) * 10.0) as i32 + 20;
                let height = (height as usize).clamp(0, CHUNK_HEIGHT - 1);

                // Top layer is grass
                chunk[VoxelPosition::new(x, height, z)] = VoxelType::Grass;
                // Three layers of dirt
                for y in (height - 3).max(0)..height {
                    chunk[VoxelPosition::new(x, y, z)] = VoxelType::Dirt;
                }
                // Below that is stone
                for y in 0..(height - 3).max(0) {
                    chunk[VoxelPosition::new(x, y, z)] = VoxelType::Stone;
                }
            }
        }

        self.chunks.insert(position, chunk);
        entity
    }

    pub fn set_voxel(&mut self, commands: &mut Commands, pos: WorldPosition, voxel: VoxelType) {
        let voxel_pos = world_to_voxel_position(pos);
        let chunk_pos = world_to_chunk_position(pos);
        let chunk = self.chunks.get_mut(&chunk_pos).unwrap();
        chunk[voxel_pos] = voxel;
        commands.entity(chunk.entity()).insert(NeedsRemesh);

        // Voxels on a chunk border influence the faces of the adjacent chunk
        if voxel_pos.x == 0 {
            if let Some(neighbor) = self.get_chunk_neighbor(chunk_pos, -1, 0) {
                commands.entity(neighbor.entity()).insert(NeedsRemesh);
            }
        } else if voxel_pos.x == CHUNK_WIDTH - 1 {
            if let Some(neighbor) = self.get_chunk_neighbor(chunk_pos, 1, 0) {
                commands.entity(neighbor.entity()).insert(NeedsRemesh);
            }
        }
        if voxel_pos.z == 0 {
            if let Some(neighbor) = self.get_chunk_neighbor(chunk_pos, 0, -1) {
                commands.entity(neighbor.entity()).insert(NeedsRemesh);
            }
        } else if voxel_pos.z == CHUNK_WIDTH - 1 {
            if let Some(neighbor) = self.get_chunk_neighbor(chunk_pos, 0, 1) {
                commands.entity(neighbor.entity()).insert(NeedsRemesh);
            }
        }
    }

    pub fn get_chunk_neighbor(&self, position: ChunkPosition, dx: i32, dz: i32) -> Option<&ChunkData> {
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

/// Builds (or rebuilds) the mesh of every chunk marked with [`NeedsRemesh`].
pub fn remesh_chunks(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunks: Query<(Entity, &Chunk, Option<&Mesh3d>), With<NeedsRemesh>>,
) {
    for (entity, chunk, mesh3d) in &chunks {
        let Some(data) = world.chunks.get(&chunk.position) else {
            continue;
        };

        let mesh = data.build_mesh(&world);

        match mesh3d {
            Some(handle) => {
                let _ = meshes.insert(&handle.0, mesh);
            }
            None => {
                commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
            }
        }
        commands.entity(entity).remove::<NeedsRemesh>();
    }
}

pub fn despawn_retired_chunks(
    mut commands: Commands,
    retired_chunks: Query<(Entity, &Chunk), With<NeedsDespawn>>,
) {
    for (entity, chunk) in &retired_chunks {
        log::info!("Despawning chunk ({}, {})", chunk.position.x, chunk.position.z);
        commands.entity(entity).despawn();
    }
}

