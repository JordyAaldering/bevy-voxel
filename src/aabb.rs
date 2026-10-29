use crate::coordinates::WorldCoord;

pub struct Aabb {
    pub min: WorldCoord,
    pub max: WorldCoord,
}
