mod face;

use std::ops::{Index, IndexMut};

use bevy::{asset::RenderAssetUsages, input::mouse::AccumulatedMouseMotion, mesh::{Indices, PrimitiveTopology}, platform::collections::HashMap, prelude::*, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};
use bevy_inspector_egui::{bevy_egui::{EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext}, egui, prelude::*, quick::{ResourceInspectorPlugin, WorldInspectorPlugin}};

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
    voxels: [VoxelType; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ChunkPosition {
    x: i32,
    z: i32,
}

impl ChunkPosition {
    fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    fn world_position(self) -> Vec3 {
        vec3(
            self.x as f32 * CHUNK_WIDTH as f32,
            0.0,
            self.z as f32 * CHUNK_WIDTH as f32,
        )
    }
}

#[derive(Default, Resource)]
struct World {
    chunks: HashMap<ChunkPosition, Chunk>,
}

impl World {
    fn try_get_neighbor_chunk(&self, position: ChunkPosition, dx: i32, dz: i32) -> Option<&Chunk> {
        let neighbor_position = ChunkPosition::new(position.x + dx,position.z + dz);
        self.chunks.get(&neighbor_position)
    }
}

#[derive(Default)]
struct MeshData {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VoxelType {
    Air,
    Grass,
}

impl VoxelType {
    fn is_transparent(&self) -> bool {
        match self {
            Self::Air => true,
            Self::Grass => false,
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self { voxels: [VoxelType::Air; _] }
    }
}

impl Chunk {
    fn xyz_to_index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < CHUNK_WIDTH);
        x + CHUNK_WIDTH * (y + CHUNK_HEIGHT * z)
    }

    fn index_to_xyz(idx: usize) -> (usize, usize, usize) {
        let x = idx % CHUNK_WIDTH;
        let y = (idx / CHUNK_WIDTH) % CHUNK_HEIGHT;
        let z = idx / (CHUNK_WIDTH * CHUNK_HEIGHT);
        (x, y, z)
    }

    fn try_get_neighbor_voxel(&self, x: usize, y: usize, z: usize, direction: FaceDirection) -> Option<VoxelType> {
        let (nx, ny, nz) = match direction {
            FaceDirection::Right => (x + 1, y, z),
            FaceDirection::Left => (x.wrapping_sub(1), y, z),
            FaceDirection::Top => (x, y + 1, z),
            FaceDirection::Bottom => (x, y.wrapping_sub(1), z),
            FaceDirection::Front => (x, y, z + 1),
            FaceDirection::Back => (x, y, z.wrapping_sub(1)),
        };

        if nx < CHUNK_WIDTH && ny < CHUNK_HEIGHT && nz < CHUNK_WIDTH {
            let idx = Self::xyz_to_index(nx, ny, nz);
            Some(self.voxels[idx])
        } else {
            None
        }
    }

    fn iter(&self) -> impl Iterator<Item = (usize, usize, usize, VoxelType)> {
        (0..CHUNK_WIDTH).flat_map(move |z| {
            (0..CHUNK_HEIGHT).flat_map(move |y| {
                (0..CHUNK_WIDTH).map(move |x| {
                    let idx = Self::xyz_to_index(x, y, z);
                    let voxel = self.voxels[idx];
                    (x, y, z, voxel)
                })
            })
        })
    }
}

impl Index<(usize, usize, usize)> for Chunk {
    type Output = VoxelType;

    fn index(&self, (x, y, z): (usize, usize, usize)) -> &Self::Output {
        let idx = Self::xyz_to_index(x, y, z);
        &self.voxels[idx]
    }
}

impl IndexMut<(usize, usize, usize)> for Chunk {
    fn index_mut(&mut self, (x, y, z): (usize, usize, usize)) -> &mut Self::Output {
        let idx = Self::xyz_to_index(x, y, z);
        &mut self.voxels[idx]
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
        .init_resource::<World>()
        .init_resource::<CameraSettings>()
        .register_type::<CameraSettings>()
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(ResourceInspectorPlugin::<CameraSettings>::default())
        .add_systems(EguiPrimaryContextPass, chunk_debug_ui)
        .add_systems(Startup, (
            spawn_global_illumination,
            spawn_camera,
            spawn_world,
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

fn chunk_debug_ui(mut contexts: EguiContexts, world: Res<World>) -> Result {
    egui::Window::new("Chunks").show(contexts.ctx_mut()?, |ui| {
        ui.label(format!("Loaded chunks: {}", world.chunks.len()));

        ui.separator();

        for (position, chunk) in &world.chunks {
            let solid_voxels = chunk.iter().filter(|(_, _, _, voxel)| !voxel.is_transparent()).count();

            ui.collapsing(
                format!("Chunk ({}, {})", position.x, position.z),
               |ui| {
                    ui.label(format!("Solid voxels: {}", solid_voxels));
                    ui.label(format!(
                        "World position: ({}, {})",
                        position.x * CHUNK_WIDTH as i32,
                        position.z * CHUNK_WIDTH as i32,
                    ));
                },
            );
        }
    });

    Ok(())
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<World>,
) {
    let material = materials.add(
        Color::linear_rgb(0.2, 0.7, 0.3)
    );

    // We need to instantiate all chunks first
    for z in -1..=1 {
        for x in -1..=1 {
            let position = ChunkPosition::new(x, z);
            let chunk = generate_chunk(position);
            world.chunks.insert(position, chunk);
        }
    }

    // Now we can generate the meshes
    for (&pos, chunk) in &world.chunks {
        commands.spawn((
            Transform::from_translation(pos.world_position()),
            Mesh3d(meshes.add(generate_chunk_mesh(&world, chunk, pos))),
            MeshMaterial3d(material.clone()),
        ));
    }
}

fn generate_chunk(pos: ChunkPosition) -> Chunk {
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_WIDTH {
        for y in 0..(((pos.x + pos.z) as usize % CHUNK_HEIGHT) + 1) {
            for z in 0..CHUNK_WIDTH {
                chunk[(x, y, z)] = VoxelType::Grass;
            }
        }
    }

    chunk
}

fn generate_chunk_mesh(world: &World, chunk: &Chunk, chunk_pos: ChunkPosition) -> Mesh {
    let mut mesh = MeshData::default();

    for (x, y, z, voxel) in chunk.iter() {
        if voxel == VoxelType::Air {
            continue;
        }

        let origin = vec3(x as f32, y as f32, z as f32);

        for face in [
            FaceDirection::Right,
            FaceDirection::Left,
            FaceDirection::Top,
            FaceDirection::Bottom,
            FaceDirection::Front,
            FaceDirection::Back,
        ] {
            if let Some(neighbor) = chunk.try_get_neighbor_voxel(x, y, z, face) {
                if !neighbor.is_transparent() {
                    continue;
                }
            } else {
                // If chunk.try_get_neighbor_voxel returns None, it means that the neighbor is out of bounds.
                // In that case, we need to check the neighboring chunk in the world
                let (dx, dz) = match face {
                    FaceDirection::Right => (1, 0),
                    FaceDirection::Left => (-1, 0),
                    FaceDirection::Front => (0, 1),
                    FaceDirection::Back => (0, -1),
                    // We are at the build limit, so we always have to show this face
                    FaceDirection::Top => (0, 0),
                    FaceDirection::Bottom => (0, 0),
                };

                if dx != 0 || dz != 0 {
                    if let Some(neighbor_chunk) = world.try_get_neighbor_chunk(chunk_pos, dx, dz) {
                        let (nx, ny, nz) = match face {
                            FaceDirection::Right => (0, y, z),
                            FaceDirection::Left => (CHUNK_WIDTH - 1, y, z),
                            FaceDirection::Front => (x, y, 0),
                            FaceDirection::Back => (x, y, CHUNK_WIDTH - 1),
                            FaceDirection::Top => unreachable!(),
                            FaceDirection::Bottom => unreachable!(),
                        };
                        if !neighbor_chunk[(nx, ny, nz)].is_transparent() {
                            continue;
                        }
                    } else {
                        // No neighboring chunk, so assume that we don't need to render this face
                        continue;
                    }
                }
            }

            let face = Face::new(origin, face);

            let start = mesh.vertices.len() as u32;
            mesh.vertices.extend_from_slice(&face.vertices());
            mesh.indices.extend_from_slice(&face.indices(start));
            mesh.normals.extend_from_slice(&face.normals());
            mesh.uvs.extend_from_slice(&face.uvs());
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh.uvs)
        .with_inserted_indices(Indices::U32(mesh.indices))
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
