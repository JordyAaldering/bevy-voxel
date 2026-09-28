mod voxel;

use bevy::{asset::RenderAssetUsages, input::mouse::AccumulatedMouseMotion, mesh::{Indices, PrimitiveTopology}, prelude::*, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};
use bevy_inspector_egui::{bevy_egui::{EguiContext, EguiPlugin, PrimaryEguiContext}, prelude::*, quick::ResourceInspectorPlugin};

use crate::voxel::Voxel;

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
        .add_plugins(ResourceInspectorPlugin::<CameraSettings>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_cursor, move_camera))
        .run();
}

fn generate_voxel_mesh(voxel: &Voxel) -> Mesh {
    let x = voxel.x as f32;
    let y = voxel.y as f32;
    let z = voxel.z as f32;

    let positions: Vec<[f32; 3]> = vec![
        // Right
        [x + 1.0, y,       z      ],
        [x + 1.0, y,       z + 1.0],
        [x + 1.0, y + 1.0, z + 1.0],
        [x + 1.0, y + 1.0, z      ],
        // Left
        [x, y,       z + 1.0],
        [x, y,       z      ],
        [x, y + 1.0, z      ],
        [x, y + 1.0, z + 1.0],
        // Top
        [x,       y + 1.0, z      ],
        [x + 1.0, y + 1.0, z      ],
        [x + 1.0, y + 1.0, z + 1.0],
        [x,       y + 1.0, z + 1.0],
        // Bottom
        [x,       y, z + 1.0],
        [x + 1.0, y, z + 1.0],
        [x + 1.0, y, z      ],
        [x,       y, z      ],
        // Front
        [x + 1.0, y,       z + 1.0],
        [x,       y,       z + 1.0],
        [x,       y + 1.0, z + 1.0],
        [x + 1.0, y + 1.0, z + 1.0],
        // Back
        [x,       y,       z],
        [x + 1.0, y,       z],
        [x + 1.0, y + 1.0, z],
        [x,       y + 1.0, z],
    ];

    let normals: Vec<[f32; 3]> = [
        [ 1.0,  0.0,  0.0], // Right
        [-1.0,  0.0,  0.0], // Left
        [ 0.0,  1.0,  0.0], // Top
        [ 0.0, -1.0,  0.0], // Bottom
        [ 0.0,  0.0,  1.0], // Front
        [ 0.0,  0.0, -1.0], // Back
    ]
        .into_iter()
        .flat_map(|normal| [normal, normal, normal, normal])
        .collect();

    let indices: Vec<u32> = vec![
         0,  1,  2,  0,  2,  3, // Right
         4,  5,  6,  4,  6,  7, // Left
         8,  9, 10,  8, 10, 11, // Top
        12, 13, 14, 12, 14, 15, // Bottom
        16, 17, 18, 16, 18, 19, // Front
        20, 21, 22, 20, 22, 23, // Back
    ];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Voxel
    let voxel = Voxel::from_xyz(0, 0, 0);
    commands.spawn((
        voxel.transform(),
        Mesh3d(meshes.add(generate_voxel_mesh(&voxel))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.7, 0.3))),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(
            Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)
        ),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn toggle_cursor(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut egui_context: Single<&mut EguiContext, With<PrimaryEguiContext>>,
) {
    // Release the cursor when escape is pressed
    if keyboard.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    // Recapture when a click occurs outside egui
    if cursor.grab_mode == CursorGrabMode::None
        && mouse.just_pressed(MouseButton::Left)
        && !egui_context.get_mut().egui_wants_pointer_input()
    {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn move_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    settings: Res<CameraSettings>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    // Don't move the camera if app is unfocused
    if cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    // Look
    let delta = mouse_motion.delta;

    if delta != Vec2::ZERO {
        camera.rotate_y(-delta.x * settings.mouse_sensitivity);

        camera.rotate_local_x(-delta.y * settings.mouse_sensitivity);
    }

    // Move
    let mut movement = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        movement += *camera.forward();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement += *camera.back();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement += *camera.left();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement += *camera.right();
    }
    if keyboard.pressed(KeyCode::Space) {
        movement += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        movement -= Vec3::Y;
    }

    if movement != Vec3::ZERO {
        camera.translation += settings.move_speed * time.delta_secs() * movement.normalize();
    }
}
