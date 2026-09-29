mod face;

use bevy::{asset::RenderAssetUsages, input::mouse::AccumulatedMouseMotion, mesh::{Indices, PrimitiveTopology}, prelude::*, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};
use bevy_inspector_egui::{bevy_egui::{EguiContext, EguiPlugin, PrimaryEguiContext}, prelude::*, quick::{ResourceInspectorPlugin, WorldInspectorPlugin}};

use crate::face::{Face, FaceDirection};

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

const CHUNK_WIDTH: usize = 3;
const CHUNK_HEIGHT: usize = 4;

struct Chunk {
    voxels: [Voxel; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH],
}

#[derive(Default)]
struct MeshData {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Voxel {
    Air,
    Grass,
}

impl Default for Chunk {
    fn default() -> Self {
        Self { voxels: [Voxel::Air; _] }
    }
}

impl Chunk {
    fn get_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Voxel {
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < CHUNK_WIDTH);
        let idx = x + CHUNK_WIDTH * (y + CHUNK_HEIGHT * z);
        &mut self.voxels[idx]
    }

    fn iter(&self) -> impl Iterator<Item = (usize, usize, usize, Voxel)> {
        let mut idx = 0;
        (0..CHUNK_WIDTH).flat_map(move |z| {
            (0..CHUNK_HEIGHT).flat_map(move |y| {
                (0..CHUNK_WIDTH).map(move |x| {
                    let voxel = self.voxels[idx];
                    idx += 1;
                    (x, y, z, voxel)
                })
            })
        })
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

fn spawn_global_illumination(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(
            Quat::from_euler(EulerRot::XYZ, 6.0, -7.5, 0.0)
        ),
    ));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.5, 2.0, 4.0)
            .looking_at(vec3(0.5, 0.5, 0.5), Vec3::Y),
    ));
}

fn spawn_voxel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_WIDTH {
        for y in 0..(CHUNK_HEIGHT / 2) {
            for z in 0..CHUNK_WIDTH {
                let voxel = chunk.get_mut(x, y, z);
                *voxel = Voxel::Grass;
            }
        }
    }

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(generate_chunk_mesh(&chunk))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.7, 0.3))),
    ));
}

fn generate_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut mesh = MeshData::default();

    for (x, y, z, voxel) in chunk.iter() {
        generate_voxel_mesh(&mut mesh, vec3(x as f32, y as f32, z as f32), voxel);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh.uvs)
        .with_inserted_indices(Indices::U32(mesh.indices))
}

fn generate_voxel_mesh(mesh: &mut MeshData, origin: Vec3, voxel: Voxel) {
    if voxel == Voxel::Air {
        return;
    }

    for face in [
        FaceDirection::Right,
        FaceDirection::Left,
        FaceDirection::Top,
        FaceDirection::Bottom,
        FaceDirection::Front,
        FaceDirection::Back,
    ] {
        let face = Face::new(origin, face);

        let start = mesh.vertices.len() as u32;
        mesh.vertices.extend_from_slice(&face.vertices());
        mesh.indices.extend_from_slice(&face.indices(start));
        mesh.normals.extend_from_slice(&face.normals());
        mesh.uvs.extend_from_slice(&face.uvs());
    }
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
