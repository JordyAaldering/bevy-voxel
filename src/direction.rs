use bevy::prelude::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Increasing x
    Right,
    /// Decreasing x
    Left,
    /// Increasing y
    Top,
    /// Decreasing y
    Bottom,
    /// Increasing z
    Front,
    /// Decreasing z
    Back,
}

impl Direction {
    pub const fn iter() -> [Self; 6] {
        [
            Self::Right,
            Self::Left,
            Self::Top,
            Self::Bottom,
            Self::Front,
            Self::Back,
        ]
    }

    /// Returns the outward-facing normal vector for this direction.
    pub const fn vec3(&self) -> Vec3 {
        match self {
            Self::Right  => Vec3::X,
            Self::Left   => Vec3::NEG_X,
            Self::Top    => Vec3::Y,
            Self::Bottom => Vec3::NEG_Y,
            Self::Front  => Vec3::Z,
            Self::Back   => Vec3::NEG_Z,
        }
    }
}
