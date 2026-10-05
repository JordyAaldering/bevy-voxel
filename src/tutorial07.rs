mod chunk;
mod face;
mod texture_atlas;
mod voxel;
mod world;

use bevy::{asset::RenderAssetUsages, input::mouse::AccumulatedMouseMotion, mesh::{Indices, PrimitiveTopology}, prelude::*, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};
use bevy_inspector_egui::{bevy_egui::{EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext}, egui, prelude::*, quick::{ResourceInspectorPlugin, WorldInspectorPlugin}};

use crate::{chunk::*, face::*, texture_atlas::{TextureAtlas, TextureAtlasPosition}, voxel::*, world::*};

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct CameraSettings {
    #[inspector(min = 0.1, max = 20.0)]
    move_speed: f32,
    #[inspector(min = 0.1, max = 20.0)]
    jump_strenght: f32,
    #[inspector(min = 0.0001, max = 0.01)]
    mouse_sensitivity: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            jump_strenght: 7.0,
            mouse_sensitivity: 0.002,
        }
    }
}

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_EYE_HEIGHT: f32 = 1.6;
const MAX_PITCH: f32 = 1.4;
const GRAVITY: f32 = 20.0;

#[derive(Resource, Default, PartialEq, Eq)]
enum GameMode {
    #[default]
    Survival,
    Spectator,
}

#[derive(Component)]
struct Player {
    velocity: Vec3,
    grounded: bool,
}

#[derive(Component)]
struct PlayerCamera;

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
        .init_resource::<GameMode>()
        .init_resource::<CameraSettings>()
        .register_type::<CameraSettings>()
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(ResourceInspectorPlugin::<CameraSettings>::default())
        .add_systems(EguiPrimaryContextPass, chunk_debug_ui)
        .add_systems(Startup, (
            spawn_global_illumination,
            spawn_player,
            spawn_world,
        ))
        .add_systems(Update, (
            toggle_cursor,
            toggle_gamemode,
            player_look,
            player_move,
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

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player {
                velocity: Vec3::ZERO,
                grounded: false,
            },
            Transform::from_xyz(1.5, 5.0, 1.5),
        ))
        .with_children(|player| {
            player.spawn((
                PlayerCamera,
                Camera3d::default(),
                Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 0.0)
                    //.with_rotation(rotation),
            ));
        });
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

fn toggle_gamemode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<GameMode>,
    mut player: Single<&mut Player>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    *mode = match *mode {
        GameMode::Survival => GameMode::Spectator,
        GameMode::Spectator => GameMode::Survival,
    };

    // Don't retain old vertical momentum
    player.velocity = Vec3::ZERO;
}

fn player_look(
    settings: Res<CameraSettings>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    mut player: Single<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Single<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    // Don't move the camera if app is unfocused
    if cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let delta = mouse_motion.delta;

    if delta == Vec2::ZERO {
        return;
    }

    // Horizontal movement rotates the entire player around the world's Y axis.
    player.rotate_y(-delta.x * settings.mouse_sensitivity);

    // Vertical movement rotates only the camera around its local X axis.
    let (yaw, mut pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

    pitch -= delta.y * settings.mouse_sensitivity;
    pitch = pitch.clamp(-MAX_PITCH, MAX_PITCH);

    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
}

fn player_move(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mode: Res<GameMode>,
    settings: Res<CameraSettings>,
    world: Res<VoxelWorld>,
    player: Single<(&mut Transform, &mut Player)>,
) {
    let (mut transform, mut player) = player.into_inner();

    let mut movement = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        movement += *transform.forward();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement += *transform.back();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement += *transform.left();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement += *transform.right();
    }

    let movement = movement.normalize_or_zero();

    match *mode {
        GameMode::Survival => {
            move_survival(
                &mut transform,
                &mut player,
                movement,
                &settings,
                &keyboard,
                &world,
                time.delta_secs(),
            );
        }
        GameMode::Spectator => {
            move_spectator(
                &mut transform,
                &mut player,
                movement,
                &settings,
                &keyboard,
                time.delta_secs(),
            );
        }
    }
}

fn move_survival(
    transform: &mut Transform,
    player: &mut Player,
    movement: Vec3,
    settings: &CameraSettings,
    keyboard: &ButtonInput<KeyCode>,
    world: &VoxelWorld,
    dt: f32,
) {
    player.velocity.x = movement.x * settings.move_speed;
    player.velocity.z = movement.z * settings.move_speed;

    if player.grounded && keyboard.just_pressed(KeyCode::Space) {
        player.velocity.y = settings.jump_strenght;
        player.grounded = false;
    }

    player.velocity.y -= GRAVITY * dt;

    move_with_collisions(&mut transform.translation, player, world, dt);
}

fn move_spectator(
    transform: &mut Transform,
    player: &mut Player,
    mut movement: Vec3,
    settings: &CameraSettings,
    keyboard: &ButtonInput<KeyCode>,
    dt: f32,
) {
    player.velocity = Vec3::ZERO;
    player.grounded = false;

    if keyboard.pressed(KeyCode::Space) {
        movement.y += 1.0;
    }

    if keyboard.pressed(KeyCode::ShiftLeft) {
        movement.y -= 1.0;
    }

    transform.translation += movement.normalize_or_zero() * settings.move_speed * dt;
}

fn move_with_collisions(position: &mut Vec3, player: &mut Player, world: &VoxelWorld, dt: f32) {
    player.grounded = false;
    move_x(position, &mut player.velocity, world, dt);
    move_y(position, &mut player.velocity, &mut player.grounded, world, dt);
    move_z(position, &mut player.velocity, world, dt);
}

fn move_y(
    position: &mut Vec3,
    velocity: &mut Vec3,
    grounded: &mut bool,
    world: &VoxelWorld,
    dt: f32,
) {
    if velocity.y == 0.0 {
        return;
    }

    position.y += velocity.y * dt;

    let half_width = PLAYER_WIDTH / 2.0;
    const EPSILON: f32 = 0.001;

    // Horizontal range occupied by the player
    let min_x = (position.x - half_width).floor() as i32;
    let max_x = (position.x + half_width - EPSILON).floor() as i32;

    let min_z = (position.z - half_width).floor() as i32;
    let max_z = (position.z + half_width - EPSILON).floor() as i32;

    if velocity.y < 0.0 {
        // Falling. Check the voxel layer intersecting the player's feet
        let voxel_y = position.y.floor() as i32;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if world.is_solid(WorldPosition::new(x, voxel_y, z)) {
                    // Put the player's feet exactly on top of the voxel
                    position.y = voxel_y as f32 + 1.0;
                    velocity.y = 0.0;
                    *grounded = true;
                    return;
                }
            }
        }
    } else {
        // Rising. Check the voxel layer intersecting the player's head
        let top = position.y + PLAYER_HEIGHT;
        let voxel_y = (top - EPSILON).floor() as i32;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if world.is_solid(WorldPosition::new(x, voxel_y, z)) {
                    // Put the player's head exactly below the voxel
                    position.y = voxel_y as f32 - PLAYER_HEIGHT;
                    velocity.y = 0.0;
                    return;
                }
            }
        }
    }
}

fn move_x(position: &mut Vec3, velocity: &mut Vec3, world: &VoxelWorld, dt: f32) {
    if velocity.x == 0.0 {
        return;
    }

    position.x += velocity.x * dt;

    let half_width = PLAYER_WIDTH / 2.0;
    const EPSILON: f32 = 0.001;

    let min_y = position.y.floor() as i32;
    let max_y =
        (position.y + PLAYER_HEIGHT - EPSILON).floor() as i32;

    let min_z = (position.z - half_width).floor() as i32;
    let max_z =
        (position.z + half_width - EPSILON).floor() as i32;

    if velocity.x > 0.0 {
        let right = position.x + half_width;
        let voxel_x = (right - EPSILON).floor() as i32;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.is_solid(WorldPosition::new(voxel_x, y, z)) {
                    position.x = voxel_x as f32 - half_width;

                    velocity.x = 0.0;
                    return;
                }
            }
        }
    } else {
        let left = position.x - half_width;
        let voxel_x = left.floor() as i32;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.is_solid(WorldPosition::new(voxel_x, y, z)) {
                    position.x =
                        voxel_x as f32 + 1.0 + half_width;

                    velocity.x = 0.0;
                    return;
                }
            }
        }
    }
}

fn move_z(position: &mut Vec3, velocity: &mut Vec3, world: &VoxelWorld, dt: f32) {
    if velocity.z == 0.0 {
        return;
    }

    position.z += velocity.z * dt;

    let half_width = PLAYER_WIDTH / 2.0;
    const EPSILON: f32 = 0.001;

    // Vertical range occupied by the player
    let min_y = position.y.floor() as i32;
    let max_y = (position.y + PLAYER_HEIGHT - EPSILON).floor() as i32;

    // X range occupied by the player
    let min_x = (position.x - half_width).floor() as i32;
    let max_x = (position.x + half_width - EPSILON).floor() as i32;

    if velocity.z > 0.0 {
        // Moving in +Z. Check the voxel layer in front of the player
        let front = position.z + half_width;
        let voxel_z = (front - EPSILON).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if world.is_solid(WorldPosition::new(x, y, voxel_z)) {
                    // Put the player's front edge against the voxel
                    position.z = voxel_z as f32 - half_width;
                    velocity.z = 0.0;
                    return;
                }
            }
        }
    } else {
        // Moving in -Z. Check the voxel layer behind the player
        let back = position.z - half_width;
        let voxel_z = back.floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if world.is_solid(WorldPosition::new(x, y, voxel_z)) {
                    // Put the player's back edge against the voxel
                    position.z = voxel_z as f32 + 1.0 + half_width;
                    velocity.z = 0.0;
                    return;
                }
            }
        }
    }
}
