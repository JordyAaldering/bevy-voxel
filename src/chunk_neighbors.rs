use crate::{ChunkData, coordinates::ChunkCoord};

use std::{ops::Index, sync::Arc};

use bevy::{math::CompassQuadrant, platform::collections::HashMap};

pub struct ChunkNeighbors {
    pub north: Option<Arc<ChunkData>>,
    pub south: Option<Arc<ChunkData>>,
    pub east: Option<Arc<ChunkData>>,
    pub west: Option<Arc<ChunkData>>,
}

impl ChunkNeighbors {
    pub fn from(pos: ChunkCoord, chunks: &HashMap<ChunkCoord, Arc<ChunkData>>) -> Self {
        let north = chunks.get(&ChunkCoord::new(pos.x, pos.z + 1)).cloned();
        let south = chunks.get(&ChunkCoord::new(pos.x, pos.z - 1)).cloned();
        let east  = chunks.get(&ChunkCoord::new(pos.x + 1, pos.z)).cloned();
        let west  = chunks.get(&ChunkCoord::new(pos.x - 1, pos.z)).cloned();
        Self { north, east, south, west }
    }
}

impl Index<CompassQuadrant> for ChunkNeighbors {
    type Output = Option<Arc<ChunkData>>;

    fn index(&self, index: CompassQuadrant) -> &Self::Output {
        match index {
            CompassQuadrant::North => &self.north,
            CompassQuadrant::East  => &self.east,
            CompassQuadrant::South => &self.south,
            CompassQuadrant::West  => &self.west,
        }
    }
}
