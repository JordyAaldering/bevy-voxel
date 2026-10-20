mod coordinates;
mod direction;

use crate::{
    coordinates::{ChunkCoord, VoxelCoord, WorldCoord},
    direction::Direction,
};

use std::{mem, ops::{Index, IndexMut}};

use better_default::Default;
use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    input::mouse::AccumulatedMouseMotion,
    mesh::{Indices, PrimitiveTopology},
    platform::collections::HashMap,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::bevy_egui::prelude::*;
use bevy_inspector_egui::bevy_egui::input::EguiWantsInput;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const CHUNK_WIDTH: usize = 3;
const CHUNK_HEIGHT: usize = 4;

const ATLAS_ROWS: u32 = 4;
const ATLAS_COLUMNS: u32 = 4;
const ATLAS_SIZE: u32 = ATLAS_COLUMNS * ATLAS_ROWS;

/// Prevents exact boundary contact from being treated as an overlap with the adjacent voxel.
const COLLISION_EPSILON: f32 = 1e-3;

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_WIDTH_HALF: f32 = PLAYER_WIDTH * 0.5;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_EYE_HEIGHT: f32 = 1.6;
const MAX_PITCH: f32 = 1.4;
const GRAVITY: f32 = 20.0;

#[derive(Clone, Copy, Default, PartialEq, Eq, Resource)]
enum GameMode {
    #[default]
    Survival,
    Spectator,
}

#[derive(Default, Component)]
struct Player {
    velocity: Vec3,
    grounded: bool,
}

#[derive(Component)]
struct PlayerCamera;

#[derive(Default, Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct CameraSettings {
    #[default(5.0)]
    #[inspector(min = 0.1, max = 20.0)]
    move_speed: f32,
    #[default(7.0)]
    #[inspector(min = 0.1, max = 20.0)]
    jump_strenght: f32,
    #[default(0.0025)]
    #[inspector(min = 0.0001, max = 0.01)]
    mouse_sensitivity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VoxelType {
    Air,
    Grass,
    Dirt,
}

impl VoxelType {
    fn is_transparent(&self) -> bool {
        matches!(self, VoxelType::Air)
    }

    fn is_solid(&self) -> bool {
        !self.is_transparent()
    }

    fn texture(self, direction: Direction) -> usize {
        fn atlas_index(x: usize, y: usize) -> usize {
            x + (y * ATLAS_COLUMNS as usize)
        }

        match self {
            Self::Grass => {
                match direction {
                    Direction::Top => atlas_index(3, 1),
                    Direction::Bottom => atlas_index(1, 0),
                    _ => atlas_index(2, 0),
                }
            },
            Self::Dirt => atlas_index(1, 0),
            _ => unreachable!(),
        }
    }
}

struct Chunk {
    voxels: [VoxelType; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH],
}

impl Chunk {
    fn get_neighbor_voxel(&self, pos: VoxelCoord, direction: Direction) -> Option<VoxelType> {
        let VoxelCoord { x, y, z } = pos;

        // Using `wrapping_sub` ensures that the value becomes `usize::MAX`,
        // which is definitely greather than the width and height.
        let (x, y, z) = match direction {
            Direction::Right => (x + 1, y, z),
            Direction::Left => (x.wrapping_sub(1), y, z),
            Direction::Top => (x, y + 1, z),
            Direction::Bottom => (x, y.wrapping_sub(1), z),
            Direction::Front => (x, y, z + 1),
            Direction::Back => (x, y, z.wrapping_sub(1)),
        };

        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_WIDTH {
            Some(self[VoxelCoord::new(x, y, z)])
        } else {
            None
        }
    }

    fn iter(&self) -> impl Iterator<Item = (VoxelCoord, VoxelType)> {
        (0..CHUNK_WIDTH).flat_map(move |z| {
            (0..CHUNK_HEIGHT).flat_map(move |y| {
                (0..CHUNK_WIDTH).map(move |x| {
                    let pos = VoxelCoord::new(x, y, z);
                    (pos, self[pos])
                })
            })
        })
    }
}

impl Index<VoxelCoord> for Chunk {
    type Output = VoxelType;

    fn index(&self, pos: VoxelCoord) -> &Self::Output {
        &self.voxels[pos.index()]
    }
}

impl IndexMut<VoxelCoord> for Chunk {
    fn index_mut(&mut self, pos: VoxelCoord) -> &mut Self::Output {
        &mut self.voxels[pos.index()]
    }
}

struct MeshData<'a> {
    atlas: &'a TextureAtlasLayout,
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
}

impl<'a> MeshData<'a> {
    fn new(atlas: &'a TextureAtlasLayout) -> Self {
        Self {
            atlas,
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        }
    }

    fn add_quad(&mut self, direction: Direction, origin: Vec3, texture: usize) {
        let start = self.vertices.len() as u32;
        self.add_quad_indices(start);
        self.add_quad_vertices(direction, origin);
        self.add_quad_normals(direction);
        self.add_quad_uvs(texture);
    }

    /// Adds the vertex indices for a quad (two triangles).
    ///
    /// Vertices are ordered as viewed from outside the voxel:
    /// ```text
    /// 2----3
    /// | \  |
    /// |  \ |
    /// 0----1
    /// ```
    fn add_quad_indices(&mut self, start: u32) {
        self.indices.extend([
            // First triangle
            start + 0, // Bottom left
            start + 1, // Bottom right
            start + 2, // Top left
            // Second triangle
            start + 2, // Top left
            start + 1, // Bottom right
            start + 3, // Top right
        ]);
    }

    /// Returns the four vertices of the quad.
    ///
    /// The quad is defined by the origin and direction of the face:
    /// ```text
    ///     (0,1,0)-------------------(1,1,0)
    ///          /|                  /|
    ///         / |                 / |
    ///        /  |                /  |
    /// (0,1,1)-------------(1,1,1)   |
    ///       |   |               |   |
    ///       |   |               |   |
    ///       |   |               |   |
    ///       |  (0,0,0)----------|--(1,0,0)
    ///       | /                 | /
    ///       |/                  |/
    /// (0,0,1)-------------------(1,0,1)
    /// ```
    ///
    /// Note that, in Bevy, the z axis points forward, so the going back decreases the z coordinate.
    fn add_quad_vertices(&mut self, direction: Direction, o: Vec3) {
        self.vertices.extend(match direction {
            Direction::Right => [
                o + Vec3::X + Vec3::Z,
                o + Vec3::X,
                o + Vec3::X + Vec3::Y + Vec3::Z,
                o + Vec3::X + Vec3::Y,
            ],
            Direction::Left => [
                o,
                o + Vec3::Z,
                o + Vec3::Y,
                o + Vec3::Y + Vec3::Z,
            ],
            Direction::Top => [
                o + Vec3::Y + Vec3::Z,
                o + Vec3::X + Vec3::Y + Vec3::Z,
                o + Vec3::Y,
                o + Vec3::X + Vec3::Y,
            ],
            Direction::Bottom => [
                o,
                o + Vec3::X,
                o + Vec3::Z,
                o + Vec3::X + Vec3::Z,
            ],
            Direction::Front => [
                o + Vec3::Z,
                o + Vec3::X + Vec3::Z,
                o + Vec3::Y + Vec3::Z,
                o + Vec3::X + Vec3::Y + Vec3::Z,
            ],
            Direction::Back => [
                o + Vec3::X,
                o,
                o + Vec3::X + Vec3::Y,
                o + Vec3::Y,
            ],
        });
    }

    /// Returns the normals of the 4 corners of the quad.
    fn add_quad_normals(&mut self, direction: Direction) {
        self.normals.extend([direction.normal(); 4]);
    }

    /// Returns the UV coordinates of the 4 corners of the quad.
    fn add_quad_uvs(&mut self, index: usize) {
        let rect = self.atlas.textures[index];
        let size = self.atlas.size.as_vec2();
        let min = rect.min.as_vec2() / size;
        let max = rect.max.as_vec2() / size;
        self.uvs.extend([
            vec2(min.x, max.y), // Bottom left
            vec2(max.x, max.y), // Bottom right
            vec2(min.x, min.y), // Top left
            vec2(max.x, min.y), // Top right
        ]);
    }

    fn build(&mut self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mem::take(&mut self.vertices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mem::take(&mut self.normals))
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mem::take(&mut self.uvs))
            .with_inserted_indices(Indices::U32(mem::take(&mut self.indices)))
    }
}

#[derive(Default, Resource)]
struct VoxelWorld {
    chunks: HashMap<ChunkCoord, Chunk>,
}

impl VoxelWorld {
    fn is_solid(&self, pos: WorldCoord) -> bool {
        self.get_voxel(pos)
            .is_some_and(|voxel| voxel.is_solid())
    }

    fn get_voxel(&self, pos: WorldCoord) -> Option<VoxelType> {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return None;
        }

        let chunk_pos = pos.chunk_coord();
        self.chunks.get(&chunk_pos).map(|chunk| {
            let voxel_pos = pos.local_coord();
            chunk[voxel_pos]
        })
    }

    fn get_chunk_neighbor(&self, pos: ChunkCoord, dx: i32, dz: i32) -> Option<&Chunk> {
        let neighbor_pos = ChunkCoord::new(pos.x + dx, pos.z + dz);
        self.chunks.get(&neighbor_pos)
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest()),
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ))
        .init_resource::<GameMode>()
        .init_resource::<VoxelWorld>()
        .init_resource::<CameraSettings>()
        .register_type::<CameraSettings>()
        .add_systems(Startup, (
            spawn_global_illumination,
            spawn_player,
            spawn_world,
        ))
        .add_systems(Update, (
            toggle_cursor,
            (toggle_gamemode, player_look)
                .run_if(is_focused),
            move_survival
                .run_if(is_focused.and_then(is_gamemode_survival)),
            move_spectator
                .run_if(is_focused.and_then(is_gamemode_spectator)),
        ))
        .add_systems(FixedUpdate, (
            player_physics
                .run_if(is_gamemode_survival),
        ))
        .run()
}

fn spawn_global_illumination(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
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

fn spawn_player(mut commands: Commands) {
    commands.spawn((
            Name::new("Player"),
            Player::default(),
            Transform::from_xyz(1.5, 5.0, 1.5),
        ))
        .with_children(|commands| {
            commands.spawn((
                PlayerCamera,
                Camera3d::default(),
                Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 0.0),
            ));
        });
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut world: ResMut<VoxelWorld>,
    asset_server: Res<AssetServer>,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(ATLAS_SIZE),
        ATLAS_COLUMNS,
        ATLAS_ROWS,
        None,
        None,
    ));
    let layout = layouts.get(&layout).unwrap();

    let texture = asset_server.load("blocks.png");

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..default()
    });

    // We need to instantiate all chunks first
    for z in -1..=1 {
        for x in -1..=1 {
            let chunk = generate_chunk(x, z);
            world.chunks.insert(ChunkCoord::new(x, z), chunk);
        }
    }

    // Now we can generate the meshes
    for (&chunk_pos, chunk) in &world.chunks {
        commands.spawn((
            Transform::from_translation(chunk_pos.position()),
            Mesh3d(meshes.add(build_mesh(chunk, chunk_pos, &world, layout))),
            MeshMaterial3d(material.clone()),
        ));
    }
}

fn generate_chunk(chunk_x: i32, chunk_z: i32) -> Chunk {
    let mut chunk = Chunk { voxels: [VoxelType::Air; _] };

    for x in 0..CHUNK_WIDTH {
        let max_y = (chunk_x + chunk_z) as usize % CHUNK_HEIGHT;
        for y in 0..=max_y {
            for z in 0..CHUNK_WIDTH {
                let pos = VoxelCoord::new(x, y, z);
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

fn build_mesh(chunk: &Chunk, chunk_pos: ChunkCoord, world: &VoxelWorld, atlas: &TextureAtlasLayout) -> Mesh {
    let mut mesh_data = MeshData::new(atlas);

    for (voxel_pos, voxel) in chunk.iter() {
        if voxel.is_transparent() {
            continue;
        }

        for direction in [
            Direction::Right,
            Direction::Left,
            Direction::Top,
            Direction::Bottom,
            Direction::Front,
            Direction::Back,
        ] {
            if let Some(neighbor) = chunk.get_neighbor_voxel(voxel_pos, direction) {
                if !neighbor.is_transparent() {
                    continue;
                }
            } else {
                // If get_neighbor_voxel returns None, the voxel exists in a neighboring chunk
                let offset = match direction {
                    Direction::Right => Some((1, 0)),
                    Direction::Left => Some((-1, 0)),
                    Direction::Front => Some((0, 1)),
                    Direction::Back => Some((0, -1)),
                    _ => None,
                };

                if let Some((dx, dz)) = offset &&
                    let Some(neighbor_chunk) = world.get_chunk_neighbor(chunk_pos, dx, dz)
                {
                    let VoxelCoord { x, y, z } = voxel_pos;

                    let (nx, ny, nz) = match direction {
                        Direction::Right => (0, y, z),
                        Direction::Left => (CHUNK_WIDTH - 1, y, z),
                        Direction::Front => (x, y, 0),
                        Direction::Back => (x, y, CHUNK_WIDTH - 1),
                        _ => unreachable!(),
                    };

                    let voxel_pos = VoxelCoord::new(nx, ny, nz);
                    let neighbor = neighbor_chunk[voxel_pos];
                    if !neighbor.is_transparent() {
                        continue;
                    }
                }
            }

            let origin = vec3(voxel_pos.x as f32, voxel_pos.y as f32, voxel_pos.z as f32);
            let texture = voxel.texture(direction);
            mesh_data.add_quad(direction, origin, texture);
        }
    }

    mesh_data.build()
}

fn is_focused(cursor: Single<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor.grab_mode == CursorGrabMode::Locked
}

fn is_gamemode_survival(mode: Res<GameMode>) -> bool {
    *mode == GameMode::Survival
}

fn is_gamemode_spectator(mode: Res<GameMode>) -> bool {
    *mode == GameMode::Spectator
}

fn toggle_cursor(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    egui_wants_input: Res<EguiWantsInput>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Release the cursor when escape is pressed
    if keyboard.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
    // Recapture when a click occurs outside egui
    else if mouse.just_pressed(MouseButton::Left)
        && !egui_wants_input.wants_pointer_input()
    {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn toggle_gamemode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Single<&mut Player>,
    mut mode: ResMut<GameMode>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        *mode = match *mode {
            GameMode::Survival => GameMode::Spectator,
            GameMode::Spectator => GameMode::Survival,
        };
        // Don't retain old vertical momentum
        player.velocity = Vec3::ZERO;
    }
}

fn player_look(
    settings: Res<CameraSettings>,
    mouse: Res<AccumulatedMouseMotion>,
    mut camera: Single<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    mut player: Single<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    let delta = mouse.delta;

    if delta != Vec2::ZERO {
        // Horizontal movement rotates the entire player around the world Y axis
        player.rotate_y(-delta.x * settings.mouse_sensitivity);
        // Vertical movement rotates only the camera around its local X axis
        let (yaw, mut pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

        pitch -= delta.y * settings.mouse_sensitivity;
        pitch = pitch.clamp(-MAX_PITCH, MAX_PITCH);

        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

fn move_spectator(
    time: Res<Time>,
    settings: Res<CameraSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut Transform, &mut Player)>,
) {
    let (mut transform, mut player) = player.into_inner();

    player.velocity = Vec3::ZERO;
    player.grounded = false;

    let delta = vec3(
        f32::from(keyboard.pressed(KeyCode::KeyD)) - f32::from(keyboard.pressed(KeyCode::KeyA)),
        f32::from(keyboard.pressed(KeyCode::Space)) - f32::from(keyboard.pressed(KeyCode::ShiftLeft)),
        f32::from(keyboard.pressed(KeyCode::KeyW)) - f32::from(keyboard.pressed(KeyCode::KeyS)),
    );

    if delta != Vec3::ZERO {
        let direction = delta.x * transform.right() + delta.y * Vec3::Y + delta.z * transform.forward();
        transform.translation += settings.move_speed * time.delta_secs() * direction.normalize();
    }
}

fn move_survival(
    settings: Res<CameraSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut Player, &Transform)>,
) {
    let (mut player, transform) = player.into_inner();

    let delta = vec3(
        f32::from(keyboard.pressed(KeyCode::KeyD)) - f32::from(keyboard.pressed(KeyCode::KeyA)),
        0.0,
        f32::from(keyboard.pressed(KeyCode::KeyW)) - f32::from(keyboard.pressed(KeyCode::KeyS)),
    );

    if delta != Vec3::ZERO {
        let movement = delta.x * transform.right() + delta.z * transform.forward();
        let movement = movement.normalize_or_zero();

        player.velocity.x = movement.x * settings.move_speed;
        player.velocity.z = movement.z * settings.move_speed;
    }

    if player.grounded && keyboard.just_pressed(KeyCode::Space) {
        player.velocity.y = settings.jump_strenght;
        player.grounded = false;
    }
}

fn player_physics(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    player: Single<(&mut Player, &mut Transform)>,
) {
    let (mut player, mut transform) = player.into_inner();

    player.velocity.y -= GRAVITY * time.delta_secs();
    player.grounded = false;

    move_x(&mut player, &mut transform.translation, &world, time.delta_secs());
    move_y(&mut player, &mut transform.translation, &world, time.delta_secs());
    move_z(&mut player, &mut transform.translation, &world, time.delta_secs());
}

fn move_y(
    player: &mut Player,
    position: &mut Vec3,
    world: &VoxelWorld,
    dt: f32,
) {
    if player.velocity.y == 0.0 {
        return;
    }

    position.y += player.velocity.y * dt;

    let min_x = (position.x - PLAYER_WIDTH_HALF).floor() as i32;
    let max_x = (position.x + PLAYER_WIDTH_HALF - COLLISION_EPSILON).floor() as i32;
    let min_z = (position.z - PLAYER_WIDTH_HALF).floor() as i32;
    let max_z = (position.z + PLAYER_WIDTH_HALF - COLLISION_EPSILON).floor() as i32;

    if player.velocity.y < 0.0 {
        // Falling
        let y = position.y.floor() as i32;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if world.is_solid(WorldCoord::new(x, y, z)) {
                    position.y = y as f32 + 1.0;
                    player.velocity.y = 0.0;
                    player.grounded = true;
                    return;
                }
            }
        }
    } else {
        // Jumping
        let top = position.y + PLAYER_HEIGHT;
        let y = (top - COLLISION_EPSILON).floor() as i32;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if world.is_solid(WorldCoord::new(x, y, z)) {
                    position.y = y as f32 - PLAYER_HEIGHT;
                    player.velocity.y = 0.0;
                    return;
                }
            }
        }
    }
}

fn move_x(
    player: &mut Player,
    position: &mut Vec3,
    world: &VoxelWorld,
    dt: f32
) {
    if player.velocity.x == 0.0 {
        return;
    }

    position.x += player.velocity.x * dt;

    let min_y = position.y.floor() as i32;
    let max_y = (position.y + PLAYER_HEIGHT - COLLISION_EPSILON).floor() as i32;
    let min_z = (position.z - PLAYER_WIDTH_HALF).floor() as i32;
    let max_z = (position.z + PLAYER_WIDTH_HALF - COLLISION_EPSILON).floor() as i32;

    if player.velocity.x > 0.0 {
        let right = position.x + PLAYER_WIDTH_HALF;
        let x = (right - COLLISION_EPSILON).floor() as i32;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.is_solid(WorldCoord::new(x, y, z)) {
                    position.x = x as f32 - PLAYER_WIDTH_HALF;
                    player.velocity.x = 0.0;
                    return;
                }
            }
        }
    } else {
        let left = position.x - PLAYER_WIDTH_HALF;
        let x = left.floor() as i32;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.is_solid(WorldCoord::new(x, y, z)) {
                    position.x = x as f32 + 1.0 + PLAYER_WIDTH_HALF;
                    player.velocity.x = 0.0;
                    return;
                }
            }
        }
    }
}

fn move_z(
    player: &mut Player,
    position: &mut Vec3,
    world: &VoxelWorld,
    dt: f32
) {
    if player.velocity.z == 0.0 {
        return;
    }

    position.z += player.velocity.z * dt;

    let min_x = (position.x - PLAYER_WIDTH_HALF).floor() as i32;
    let max_x = (position.x + PLAYER_WIDTH_HALF - COLLISION_EPSILON).floor() as i32;
    let min_y = position.y.floor() as i32;
    let max_y = (position.y + PLAYER_HEIGHT - COLLISION_EPSILON).floor() as i32;

    if player.velocity.z > 0.0 {
        let front = position.z + PLAYER_WIDTH_HALF;
        let z = (front - COLLISION_EPSILON).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if world.is_solid(WorldCoord::new(x, y, z)) {
                    // Put the player's front edge against the voxel
                    position.z = z as f32 - PLAYER_WIDTH_HALF;
                    player.velocity.z = 0.0;
                    return;
                }
            }
        }
    } else {
        let back = position.z - PLAYER_WIDTH_HALF;
        let z = back.floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if world.is_solid(WorldCoord::new(x, y, z)) {
                    // Put the player's back edge against the voxel
                    position.z = z as f32 + 1.0 + PLAYER_WIDTH_HALF;
                    player.velocity.z = 0.0;
                    return;
                }
            }
        }
    }
}
