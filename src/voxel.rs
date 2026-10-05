/// Chunk-local voxel position, with x and z in [0, CHUNK_WIDTH) and y in [0, CHUNK_HEIGHT).
pub struct VoxelPosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
