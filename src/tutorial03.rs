mod spawn;
mod input;
mod face;
mod voxel;

use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*, window::{CursorGrabMode, CursorOptions}};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, prelude::*, quick::{ResourceInspectorPlugin, WorldInspectorPlugin}};

use crate::{input::*, spawn::*, face::*, voxel::*};

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct CameraSettings {
    #[inspector(min = 0.1, max = 20.0)]
    move_speed: f32,
    #[inspector(min = 0.0001, max = 0.01)]
    mouse_sensitivity: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            mouse_sensitivity: 0.002,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_cursor_options: Some(CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<CameraSettings>()
        .register_type::<CameraSettings>()
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(ResourceInspectorPlugin::<CameraSettings>::default())
        .add_systems(Startup, (
            spawn_global_illumination,
            spawn_camera,
            spawn_voxel,
        ))
        .add_systems(Update, (
            toggle_cursor,
            move_camera,
        ))
        .run();
}

fn spawn_voxel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let voxel = Voxel::from_coordinates(0, 0, 0);
    commands.spawn((
        Transform::from_translation(voxel.origin()),
        Mesh3d(meshes.add(generate_voxel_mesh(&voxel))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.7, 0.3))),
    ));
}

fn generate_voxel_mesh(voxel: &Voxel) -> Mesh {
    let mut vertices = Vec::with_capacity(6 * 4); // 4 vertices per face
    let mut indices = Vec::with_capacity(6 * 6); // 6 indices per face (2 triangles)
    let mut normals = Vec::with_capacity(6 * 4); // 4 normals per face
    let mut uvs = Vec::with_capacity(6 * 4); // 4 uvs per face

    for face in [
        FaceDirection::Right,
        FaceDirection::Left,
        FaceDirection::Top,
        FaceDirection::Bottom,
        FaceDirection::Front,
        FaceDirection::Back,
    ] {
        let face = Face::new(voxel.origin(), face);

        let start = vertices.len() as u32;
        vertices.extend_from_slice(&face.vertices());
        indices.extend_from_slice(&face.indices(start));
        normals.extend_from_slice(&face.normals());
        uvs.extend_from_slice(&face.uvs());
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}
