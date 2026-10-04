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

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_EYE_HEIGHT: f32 = 1.6;

const PLAYER_SPEED: f32 = 5.0;
const JUMP_SPEED: f32 = 7.0;
const GRAVITY: f32 = 20.0;

#[derive(Component)]
struct MainCamera;

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
    fn get_voxel(&self, x: i32, y: i32, z: i32) -> Option<VoxelType> {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return None;
        }

        let chunk_x = x.div_euclid(CHUNK_WIDTH as i32);
        let chunk_z = z.div_euclid(CHUNK_WIDTH as i32);
        let local_x = x.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let local_y = y as usize;
        let local_z = z.rem_euclid(CHUNK_WIDTH as i32) as usize;

        let chunk_position = ChunkPosition::new(chunk_x, chunk_z);
        self.chunks.get(&chunk_position)
            .map(|chunk| chunk[(local_x, local_y, local_z)])
    }

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
    Dirt,
}

struct TextureAtlas {
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
struct TextureAtlasPosition {
    x: usize,
    y: usize,
}


impl TextureAtlas {
    fn uvs(&self, tile: TextureAtlasPosition) -> [Vec2; 4] {
        let tile_width = 1.0 / self.width as f32;
        let tile_height = 1.0 / self.height as f32;
        let x = tile.x as f32 * tile_width;
        let y = tile.y as f32 * tile_height;
        // Note that the Y axis is flipped in UV coordinates, so we need to flip the Y coordinate of the tile position
        [
            vec2(x, y + tile_height), // Bottom left
            vec2(x + tile_width, y + tile_height), // Bottom right
            vec2(x, y), // Top left
            vec2(x + tile_width, y), // Top right
        ]
    }
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
                    FaceDirection::Top => TextureAtlasPosition { x: 3, y: 1 },
                    FaceDirection::Bottom => TextureAtlasPosition { x: 1, y: 0 },
                    _ => TextureAtlasPosition { x: 2, y: 0 },
                }
            },
            Self::Dirt => TextureAtlasPosition { x: 1, y: 0 },
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
        .init_resource::<World>()
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
            look,
            move_freecam,
            move_player,
            follow_player,
            toggle_control_mode,
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
    commands.spawn((
        Player {
            velocity: Vec3::ZERO,
            grounded: false,
        },
        Transform::from_xyz(1.5, 5.0, 1.5),
    ));

    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(1.5, 5.0 + PLAYER_EYE_HEIGHT, 1.5),
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
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<World>,
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
        let max_y = (pos.x + pos.z) as usize % CHUNK_HEIGHT;
        for y in 0..=max_y {
            for z in 0..CHUNK_WIDTH {
                if y == max_y {
                    chunk[(x, y, z)] = VoxelType::Grass;
                } else {
                    chunk[(x, y, z)] = VoxelType::Dirt;
                }
            }
        }
    }

    chunk
}

fn generate_chunk_mesh(world: &World, chunk: &Chunk, chunk_pos: ChunkPosition) -> Mesh {
    let mut mesh = MeshData::default();

    let atlas = TextureAtlas { width: 4, height: 4 };

    for (x, y, z, voxel) in chunk.iter() {
        if voxel == VoxelType::Air {
            continue;
        }

        let origin = vec3(x as f32, y as f32, z as f32);

        for direction in [
            FaceDirection::Right,
            FaceDirection::Left,
            FaceDirection::Top,
            FaceDirection::Bottom,
            FaceDirection::Front,
            FaceDirection::Back,
        ] {
            if let Some(neighbor) = chunk.try_get_neighbor_voxel(x, y, z, direction) {
                if !neighbor.is_transparent() {
                    continue;
                }
            } else {
                // If chunk.try_get_neighbor_voxel returns None, it means that the neighbor is out of bounds.
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
                    if let Some(neighbor_chunk) = world.try_get_neighbor_chunk(chunk_pos, dx, dz) {
                        let (nx, ny, nz) = match direction {
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

fn look(
    settings: Res<CameraSettings>,
    mouse_motion: Res<AccumulatedMouseMotion>,
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
}

fn move_freecam(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mode: Res<GameMode>,
    settings: Res<CameraSettings>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    if *mode != GameMode::Spectator {
        return;
    }

    // Don't move the camera if app is unfocused
    if cursor.grab_mode == CursorGrabMode::None {
        return;
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

fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mode: Res<GameMode>,
    world: Res<World>,
    mut player: Single<(&mut Transform, &mut Player)>,
) {
    if *mode != GameMode::Survival {
        return;
    }

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

    if movement != Vec3::ZERO {
        movement = movement.normalize();
    }

    player.velocity.x = movement.x * PLAYER_SPEED;
    player.velocity.z = movement.z * PLAYER_SPEED;

    if player.grounded && keyboard.just_pressed(KeyCode::Space) {
        player.velocity.y = JUMP_SPEED;
        player.grounded = false;
    }

    player.velocity.y -= GRAVITY * time.delta_secs();

    move_with_collisions(
        &mut transform.translation,
        &mut player,
        &world,
        time.delta_secs(),
    );
}

/// Makes the camera follow the player when in player mode.
fn follow_player(
    mode: Res<GameMode>,
    player: Single<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera: Single<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    if *mode != GameMode::Survival {
        return;
    }

    camera.translation = player.translation + Vec3::Y * PLAYER_EYE_HEIGHT;

    let forward = player.forward();
    let yaw = forward.x.atan2(-forward.z);

    let (_, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}

fn move_with_collisions(
    position: &mut Vec3,
    player: &mut Player,
    world: &World,
    dt: f32,
) {
    player.grounded = false;

    // X
    let old_x = position.x;
    position.x += player.velocity.x * dt;

    if collides(world, *position) {
        position.x = old_x;
        player.velocity.x = 0.0;
    }

    // Y
    let old_y = position.y;
    position.y += player.velocity.y * dt;

    if collides(world, *position) {
        if player.velocity.y < 0.0 {
            player.grounded = true;
        }

        position.y = old_y;
        player.velocity.y = 0.0;
    }

    // Z
    let old_z = position.z;
    position.z += player.velocity.z * dt;

    if collides(world, *position) {
        position.z = old_z;
        player.velocity.z = 0.0;
    }
}

fn toggle_control_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<GameMode>,
    player: Single<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera: Single<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    match *mode {
        GameMode::Survival => {
            *mode = GameMode::Spectator;
        }
        GameMode::Spectator => {
            *mode = GameMode::Survival;

            camera.translation =
                player.translation + Vec3::Y * PLAYER_EYE_HEIGHT;
        }
    }
}

fn collides(world: &World, position: Vec3) -> bool {
    let half_width = PLAYER_WIDTH / 2.0;

    let min = vec3(
        position.x - half_width,
        position.y,
        position.z - half_width,
    );

    let max = vec3(
        position.x + half_width,
        position.y + PLAYER_HEIGHT,
        position.z + half_width,
    );

    const EPSILON: f32 = 0.001;

    let min_x = min.x.floor() as i32;
    let max_x = (max.x - EPSILON).floor() as i32;

    let min_y = min.y.floor() as i32;
    let max_y = (max.y - EPSILON).floor() as i32;

    let min_z = min.z.floor() as i32;
    let max_z = (max.z - EPSILON).floor() as i32;

    for z in min_z..=max_z {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if world.get_voxel(x, y, z).is_some_and(|v| !v.is_transparent()) {
                    return true;
                }
            }
        }
    }

    false
}
