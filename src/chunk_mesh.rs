use std::mem;

use bevy::{asset::RenderAssetUsages, math::{Vec2, Vec3}, mesh::{Indices, Mesh, PrimitiveTopology}};

use crate::{face::Face, texture_atlas::{TextureAtlas, TextureAtlasPosition}};

pub struct MeshData {
    atlas: TextureAtlas,
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
}

impl MeshData {
    pub fn new(atlas: TextureAtlas) -> Self {
        Self {
            atlas,
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        }
    }

    pub fn build(&mut self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mem::take(&mut self.vertices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mem::take(&mut self.normals))
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mem::take(&mut self.uvs))
            .with_inserted_indices(Indices::U32(mem::take(&mut self.indices)))
    }

    pub fn add_face(&mut self, face: Face, texture: TextureAtlasPosition) {
        let start = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&face.vertices());
        self.indices.extend_from_slice(&face.indices(start));
        self.normals.extend_from_slice(&face.normals());
        self.uvs.extend_from_slice(&self.atlas.uvs(texture));
    }
}
