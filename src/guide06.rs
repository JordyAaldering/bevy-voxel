mod chunk;
mod face;
mod texture_atlas;
mod voxel;
mod world;

use bevy::{asset::RenderAssetUsages, input::mouse::AccumulatedMouseMotion, mesh::{Indices, PrimitiveTopology}, prelude::*, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};
use bevy_inspector_egui::{bevy_egui::{EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext}, egui, prelude::*, quick::{ResourceInspectorPlugin, WorldInspectorPlugin}};

use crate::{chunk::*, face::*, voxel::*, texture_atlas::{TextureAtlas, TextureAtlasPosition}, world::*};

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
            mouse_sensitivity: 0.0025,
        }
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
pub enum VoxelType {
    Air,
    Grass,
    Dirt,
}

impl VoxelType {
    fn is_transparent(&self) -> bool {
        match self {
            Self::Air => true,
            _ => false,
        }
    }

    fn is_solid(&self) -> bool {
        match self {
            Self::Air => false,
            _ => true,
        }
    }

    fn texture(self, face: FaceDirection) -> TextureAtlasPosition {
        match self {
            Self::Air => unreachable!(),
            Self::Grass => {
                match face {
                    FaceDirection::Top => TextureAtlasPosition::new(3, 1),
                    FaceDirection::Bottom => TextureAtlasPosition::new(1, 0),
                    _ => TextureAtlasPosition::new(2, 0),
                }
            },
            Self::Dirt => TextureAtlasPosition::new(1, 0),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_cursor_options: Some(CursorOptions {
                    grab_mode: CursorGrabMode::Locked,
                    visible: false,
                    ..default()
                }),
                ..default()
            })
        )
        .init_resource::<VoxelWorld>()
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
            shadow_normal_bias: 1.0,
            shadow_depth_bias: 0.05,
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

fn chunk_debug_ui(mut contexts: EguiContexts, world: Res<VoxelWorld>) -> Result {
    egui::Window::new("Chunks").show(contexts.ctx_mut()?, |ui| {
        ui.label(format!("Loaded chunks: {}", world.chunks.len()));

        ui.separator();

        for (position, chunk) in &world.chunks {
            let solid_voxels = chunk.iter().filter(|(_pos, voxel)| !voxel.is_transparent()).count();

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
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<VoxelWorld>,
) {
    let texture: Handle<Image> = asset_server.load("blocks.png");

    let material: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..default()
    });

    // We need to instantiate all chunks first
    for z in -1..=1 {
        for x in -1..=1 {
            let position = ChunkPosition::new(x, z);
            let chunk = generate_chunk(position);
            world.chunks.insert(position, chunk);
        }
    }

    // Now we can generate the meshes
    for (&chunk_pos, chunk) in &world.chunks {
        commands.spawn((
            Transform::from_translation(chunk_pos.origin()),
            Mesh3d(meshes.add(generate_chunk_mesh(&world, chunk, chunk_pos))),
            MeshMaterial3d(material.clone()),
        ));
    }
}

fn generate_chunk(pos: ChunkPosition) -> Chunk {
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_WIDTH {
        let max_y = (pos.x + pos.z) as usize % CHUNK_HEIGHT;
        for y in 0..=max_y {
            for z in 0..CHUNK_WIDTH {
                let pos = VoxelPosition::new(x, y, z);
                if y == max_y {
                    chunk[pos] = VoxelType::Grass;
                } else {
                    chunk[pos] = VoxelType::Dirt;
                }
            }
        }
    }

    chunk
}

fn generate_chunk_mesh(world: &VoxelWorld, chunk: &Chunk, chunk_pos: ChunkPosition) -> Mesh {
    let mut mesh = MeshData::default();

    let atlas = TextureAtlas::new(4, 4);

    for (pos, voxel) in chunk.iter() {
        if voxel == VoxelType::Air {
            continue;
        }

        let origin = pos.origin();

        for direction in [
            FaceDirection::Right,
            FaceDirection::Left,
            FaceDirection::Top,
            FaceDirection::Bottom,
            FaceDirection::Front,
            FaceDirection::Back,
        ] {
            if let Some(neighbor) = chunk.get_neighbor_voxel(pos, direction) {
                if !neighbor.is_transparent() {
                    continue;
                }
            } else {
                // If chunk.get_neighbor_voxel returns None, it means that the neighbor is out of bounds.
                // In that case, we need to check the neighboring chunk in the world
                let (dx, dz) = match direction {
                    FaceDirection::Right => (1, 0),
                    FaceDirection::Left => (-1, 0),
                    FaceDirection::Front => (0, 1),
                    FaceDirection::Back => (0, -1),
                    // We are at the build limit, so we always have to show this face
                    FaceDirection::Top => (0, 0),
                    FaceDirection::Bottom => (0, 0),
                };

                if dx != 0 || dz != 0 {
                    if let Some(neighbor_chunk) = world.get_chunk_neighbor(chunk_pos, dx, dz) {
                        let (x, y, z) = (pos.x, pos.y, pos.z);
                        let (nx, ny, nz) = match direction {
                            FaceDirection::Right => (0, y, z),
                            FaceDirection::Left => (CHUNK_WIDTH - 1, y, z),
                            FaceDirection::Front => (x, y, 0),
                            FaceDirection::Back => (x, y, CHUNK_WIDTH - 1),
                            FaceDirection::Top => unreachable!(),
                            FaceDirection::Bottom => unreachable!(),
                        };
                        let neighbor_pos = VoxelPosition::new(nx, ny, nz);
                        if !neighbor_chunk[neighbor_pos].is_transparent() {
                            continue;
                        }
                    } else {
                        // No neighboring chunk, so assume that we don't need to render this face
                        continue;
                    }
                }
            }

            let face = Face::new(origin, direction);
            let texture = voxel.texture(direction);

            let start = mesh.vertices.len() as u32;
            mesh.vertices.extend_from_slice(&face.vertices());
            mesh.indices.extend_from_slice(&face.indices(start));
            mesh.normals.extend_from_slice(&face.normals());
            mesh.uvs.extend_from_slice(&atlas.uvs(texture));
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
