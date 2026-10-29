use crate::{VoxelType, direction::Direction};

/// Set a coordinate to air to require that space to be cleared by the structure,
/// otherwise if there happens to be any voxels already at that position it remains.
/// For trees, for example, we don't want to clear the space below it.
/// But we do want to clear the space inside a house.
/// A prefab placed relative to an origin voxel.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Structure {
    /// Offset-from-origin voxels. Positions not listed are left untouched.
    pub voxels: Vec<StructureVoxel>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StructureVoxel {
    /// Offset relative to the structure origin (can be negative).
    pub pos: [i32; 3],
    pub voxel: VoxelType,
    /// Orientation, for voxels that have one (e.g. logs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
}
