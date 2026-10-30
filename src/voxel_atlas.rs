use std::{collections::BTreeSet, fs, io, ops::{Index, IndexMut}, path::{Path, PathBuf}, time::Duration};

use better_default::Default;
use bevy::{
    prelude::*,
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageSampler, ImageType, TextureAtlasBuilder},
    platform::collections::HashMap,
};

use crate::{VoxelType, direction::Direction};

const VOXELS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/voxels");

/// Describes which image is used on each face of a voxel, as found in `voxels.json`.
/// More specific faces take precedence: `front`/`back`/`left`/`right`, then
/// `sides` (horizontal faces only) or `top`/`bottom`, then `all`.
#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FaceSpec {
    all: Option<String>,
    top: Option<String>,
    bottom: Option<String>,
    sides: Option<String>,
    front: Option<String>,
    back: Option<String>,
    left: Option<String>,
    right: Option<String>,
}

impl FaceSpec {
    fn resolve(&self, direction: Direction) -> Option<&str> {
        match direction {
            Direction::Top    => self.top.as_ref(),
            Direction::Bottom => self.bottom.as_ref(),
            Direction::Front  => self.front.as_ref().or(self.sides.as_ref()),
            Direction::Back   => self.back.as_ref().or(self.sides.as_ref()),
            Direction::Left   => self.left.as_ref().or(self.sides.as_ref()),
            Direction::Right  => self.right.as_ref().or(self.sides.as_ref()),
        }
        .or(self.all.as_ref())
        .map(String::as_str)
    }
}

#[derive(Resource)]
pub struct VoxelAtlas {
    layout: TextureAtlasLayout,
    /// Atlas index per voxel type and direction. Air is never rendered and stays 0.
    faces: [[usize; 6]; VoxelType::ALL.len()],
}

impl VoxelAtlas {
    pub fn texture(&self, voxel: VoxelType, direction: Direction) -> usize {
        self.faces[voxel as usize][direction as usize]
    }

    pub fn layout(&self) -> &TextureAtlasLayout {
        &self.layout
    }
}

/// Builds the atlas from the images in `assets/voxels`, as described by `voxels.json`.
pub fn build_voxel_atlas(images: &mut Assets<Image>) -> Result<(VoxelAtlas, Image), String> {
    let json_path = Path::new(VOXELS_DIR).join("voxels.json");
    let json = fs::read_to_string(&json_path).map_err(|e| format!("{}: {e}", json_path.display()))?;
    let specs: HashMap<VoxelType, FaceSpec> = serde_json::from_str(&json)
        .map_err(|e| format!("{}: {e}", json_path.display()))?;

    // Resolve every face to an image name, remembering the unique names
    let mut names = BTreeSet::new();
    let mut resolved = Vec::new();
    for voxel in VoxelType::ALL.into_iter().filter(|v| v.is_solid()) {
        let spec = specs.get(&voxel).ok_or_else(|| format!("no entry for {voxel:?} in voxels.json"))?;
        for direction in Direction::iter() {
            let name = spec.resolve(direction)
                .ok_or_else(|| format!("{voxel:?} has no image for its {direction:?} face"))?;
            names.insert(name.to_owned());
            resolved.push((voxel, direction, name.to_owned()));
        }
    }

    let mut loaded = Vec::new();
    for name in &names {
        let path = Path::new(VOXELS_DIR).join(format!("{name}.png"));
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let image = Image::from_buffer(
            &bytes,
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::nearest(),
            RenderAssetUsages::default(),
        ).map_err(|e| format!("{}: {e}", path.display()))?;
        loaded.push((name, images.add(image)));
    }

    let mut builder = TextureAtlasBuilder::default();
    for (_, handle) in &loaded {
        builder.add_texture(Some(handle.id()), images.get(handle).unwrap());
    }
    let (layout, sources, image) = builder.build().map_err(|e| format!("failed to pack atlas: {e}"))?;

    let indices: HashMap<&str, usize> = loaded.iter()
        .map(|(name, handle)| (name.as_str(), sources.texture_index(handle.id()).unwrap()))
        .collect();

    let mut faces = [[0; 6]; VoxelType::ALL.len()];
    for (voxel, direction, name) in resolved {
        faces[voxel as usize][direction as usize] = indices[name.as_str()];
    }

    Ok((VoxelAtlas { layout, faces }, image))
}
