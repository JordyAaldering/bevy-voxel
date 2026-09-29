mod spawn;
mod input;

use bevy::{prelude::*, window::{CursorGrabMode, CursorOptions}};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::{ResourceInspectorPlugin, WorldInspectorPlugin}};

use crate::{input::*, spawn::*};

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
            spawn_cuboid,
        ))
        .add_systems(Update, (
            toggle_cursor,
            move_camera,
        ))
        .run();
}

fn spawn_cuboid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Transform::from_xyz(0.5, 0.5, 0.5),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.7, 0.3))),
    ));
}
