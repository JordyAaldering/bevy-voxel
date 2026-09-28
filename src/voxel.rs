use bevy::transform::components::Transform;

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

    pub fn transform(&self) -> Transform {
        Transform::from_xyz(self.x as f32, self.y as f32, self.z as f32)
    }
}
