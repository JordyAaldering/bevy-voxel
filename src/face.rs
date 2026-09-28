use bevy::math::{Vec2, Vec3, vec2, vec3};

pub struct Face {
    origin: Vec3,
    direction: FaceDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FaceDirection {
    Right,
    Left,
    Top,
    Bottom,
    Front,
    Back,
}

impl Face {
    pub fn new(origin: Vec3, direction: FaceDirection) -> Self {
        Self { origin, direction }
    }

    /// Returns the four vertices of the quad.
    ///
    /// Vertices are ordered as viewed from outside the voxel:
    /// ```text
    /// 2-----3
    /// |     |
    /// |     |
    /// 0-----1
    /// ```
    pub fn vertices(&self) -> [Vec3; 4] {
        let o = self.origin;

        match self.direction {
            FaceDirection::Right => [
                o + Vec3::X,
                o + Vec3::X + Vec3::Z,
                o + Vec3::X + Vec3::Y,
                o + Vec3::X + Vec3::Y + Vec3::Z,
            ],
            FaceDirection::Left => [
                o + Vec3::Z,
                o,
                o + Vec3::Y + Vec3::Z,
                o + Vec3::Y,
            ],
            FaceDirection::Top => [
                o + Vec3::Y,
                o + Vec3::X + Vec3::Y,
                o + Vec3::Y + Vec3::Z,
                o + Vec3::X + Vec3::Y + Vec3::Z,
            ],
            FaceDirection::Bottom => [
                o + Vec3::Z,
                o + Vec3::X + Vec3::Z,
                o,
                o + Vec3::X,
            ],
            FaceDirection::Front => [
                o,
                o + Vec3::X,
                o + Vec3::Y,
                o + Vec3::X + Vec3::Y,
            ],
            FaceDirection::Back => [
                o + Vec3::X + Vec3::Z,
                o + Vec3::Z,
                o + Vec3::X + Vec3::Y + Vec3::Z,
                o + Vec3::Y + Vec3::Z,
            ],
        }
    }

    /// Returns the vertex indices for the quad (two triangles).
    pub fn indices(&self, start: u32) -> [u32; 6] {
        [start, start + 1, start + 2, start + 2, start + 1, start + 3]
    }

    /// Returns the normals of the 4 corners of the quad.
    pub fn normals(&self) -> [Vec3; 4] {
        [self.direction.normal(); 4]
    }

    /// Returns the UV coordinates of the 4 corners of the quad.
    pub fn uvs(&self) -> [Vec2; 4] {
        [
            vec2(0.0, 0.0), // Bottom left
            vec2(1.0, 0.0), // Bottom right
            vec2(0.0, 1.0), // Top left
            vec2(1.0, 1.0), // Top right
        ]
    }
}

impl FaceDirection {
    pub fn normal(&self) -> Vec3 {
        match self {
            Self::Right  => vec3( 1.0, 0.0, 0.0),
            Self::Left   => vec3(-1.0, 0.0, 0.0),
            Self::Top    => vec3(0.0,  1.0, 0.0),
            Self::Bottom => vec3(0.0, -1.0, 0.0),
            Self::Front  => vec3(0.0, 0.0,  1.0),
            Self::Back   => vec3(0.0, 0.0, -1.0),
        }
    }
}
