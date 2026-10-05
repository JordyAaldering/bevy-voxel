use bevy::math::{Vec2, vec2};

pub struct TextureAtlas {
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
pub struct TextureAtlasPosition {
    x: usize,
    y: usize,
}

impl TextureAtlas {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn uvs(&self, tile: TextureAtlasPosition) -> [Vec2; 4] {
        let tile_width = 1.0 / self.width as f32;
        let tile_height = 1.0 / self.height as f32;
        let x = tile.x as f32 * tile_width;
        let y = tile.y as f32 * tile_height;
        // Note that the Y axis is flipped in UV coordinates, so we need to flip the Y coordinate of the tile position
        [
            vec2(x, y + tile_height),              // Bottom left
            vec2(x + tile_width, y + tile_height), // Bottom right
            vec2(x, y),                            // Top left
            vec2(x + tile_width, y),               // Top right
        ]
    }
}

impl TextureAtlasPosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}
