use crate::{VoxelType, aabb::Aabb, coordinates::WorldCoord, direction::Direction};

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

impl Structure {
    /// Bounding box of this structure.
    pub fn aabb(&self) -> Aabb {
        let mut min = WorldCoord::ZERO;
        let mut max = WorldCoord::ZERO;

        for voxel in &self.voxels {
            let WorldCoord { x, y, z } = voxel.pos;

            min.x = min.x.min(x);
            min.y = min.y.min(y);
            min.z = min.z.min(z);

            max.x = max.x.max(x);
            max.y = max.y.max(y);
            max.z = max.z.max(z);
        }

        Aabb { min, max }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StructureVoxel {
    /// Offset relative to the structure origin (can be negative).
    pub pos: WorldCoord,
    pub voxel: VoxelType,
    /// Orientation, for voxels that have one (e.g. logs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
}
