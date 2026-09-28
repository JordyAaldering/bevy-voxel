use bevy::math::Vec3;

#[derive(Clone, Debug)]
pub struct Voxel {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl Voxel {
    pub fn from_xyz(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    /// Front-bottom-left corner of the voxel: (0,0,0) in this voxel's local space.
    pub fn origin(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}
