use std::ops::{Index, IndexMut};

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

    fn texture(self, face: Direction) -> usize {
        fn atlas_index(x: usize, y: usize) -> usize {
            x + (y * ATLAS_COLUMNS as usize)
        }

        match self {
            Self::Grass => {
                match face {
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

fn atlas_uvs(layout: &TextureAtlasLayout, index: usize) -> [Vec2; 4] {
    let rect = layout.textures[index];
    let size = layout.size.as_vec2();
    let min = rect.min.as_vec2() / size;
    let max = rect.max.as_vec2() / size;
    [
        vec2(min.x, max.y), // Bottom left
        vec2(max.x, max.y), // Bottom right
        vec2(min.x, min.y), // Top left
        vec2(max.x, min.y), // Top right
    ]
}

#[derive(Clone, Copy)]
struct VoxelPosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl VoxelPosition {
    fn index(self) -> usize {
        self.x + CHUNK_WIDTH * (self.y + CHUNK_HEIGHT * self.z)
    }
}

struct Face {
    direction: Direction,
}

impl Face {
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
    fn vertices(&self, o: Vec3) -> [Vec3; 4] {
        match self.direction {
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
        }
    }

    /// Returns the vertex indices for the quad (two triangles).
    ///
    /// Vertices are ordered as viewed from outside the voxel:
    /// ```text
    /// 2----3
    /// | \  |
    /// |  \ |
    /// 0----1
    /// ```
    fn indices(&self, start: u32) -> [u32; 6] {
        [
            // First triangle
            start + 0, // Bottom left
            start + 1, // Bottom right
            start + 2, // Top left
            // Second triangle
            start + 2, // Top left
            start + 1, // Bottom right
            start + 3, // Top right
        ]
    }

    /// Returns the normals of the 4 corners of the quad.
    fn normals(&self) -> [Vec3; 4] {
        [self.direction.normal(); 4]
    }
}

#[derive(Clone, Copy)]
enum Direction {
    /// Increasing x
    Right,
    /// Decreasing x
    Left,
    /// Increasing y
    Top,
    /// Decreasing y
    Bottom,
    /// Increasing z
    Front,
    /// Decreasing z
    Back,
}

impl Direction {
    /// Returns an outward-facing normal vector for the face direction.
    fn normal(&self) -> Vec3 {
        match self {
            Self::Right  => Vec3::X,
            Self::Left   => Vec3::NEG_X,
            Self::Top    => Vec3::Y,
            Self::Bottom => Vec3::NEG_Y,
            Self::Front  => Vec3::Z,
            Self::Back   => Vec3::NEG_Z,
        }
    }
}

struct Chunk {
    voxels: [VoxelType; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH],
}

impl Chunk {
    fn get_neighbor_voxel(&self, pos: VoxelPosition, direction: Direction) -> Option<VoxelType> {
        let VoxelPosition { x, y, z } = pos;

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
            Some(self[VoxelPosition { x, y, z }])
        } else {
            None
        }
    }

    fn iter(&self) -> impl Iterator<Item = (VoxelPosition, VoxelType)> {
        (0..CHUNK_WIDTH).flat_map(move |z| {
            (0..CHUNK_HEIGHT).flat_map(move |y| {
                (0..CHUNK_WIDTH).map(move |x| {
                    let pos = VoxelPosition { x, y, z };
                    (pos, self[pos])
                })
            })
        })
    }
}

impl Index<VoxelPosition> for Chunk {
    type Output = VoxelType;

    fn index(&self, pos: VoxelPosition) -> &Self::Output {
        &self.voxels[pos.index()]
    }
}

impl IndexMut<VoxelPosition> for Chunk {
    fn index_mut(&mut self, pos: VoxelPosition) -> &mut Self::Output {
        &mut self.voxels[pos.index()]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ChunkPosition {
    x: i32,
    z: i32,
}

impl ChunkPosition {
    /// This chunk's origin in world space.
    fn origin(self) -> Vec3 {
        vec3(
            self.x as f32 * CHUNK_WIDTH as f32,
            0.0,
            self.z as f32 * CHUNK_WIDTH as f32,
        )
    }
}

#[derive(Default, Resource)]
struct VoxelWorld {
    chunks: HashMap<ChunkPosition, Chunk>,
}

impl VoxelWorld {
    pub fn is_solid(&self, pos: WorldPosition) -> bool {
        self.get_voxel(pos)
            .is_some_and(|voxel| voxel.is_solid())
    }

    pub fn get_voxel(&self, pos: WorldPosition) -> Option<VoxelType> {
        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return None;
        }

        let chunk_pos = pos.to_chunk_position();
        self.chunks.get(&chunk_pos).map(|chunk| {
            let voxel_pos = pos.to_voxel_position();
            chunk[voxel_pos]
        })
    }

    fn get_chunk_neighbor(&self, pos: ChunkPosition, dx: i32, dz: i32) -> Option<&Chunk> {
        let neighbor_pos = ChunkPosition { x: pos.x + dx, z: pos.z + dz };
        self.chunks.get(&neighbor_pos)
    }
}

#[derive(Clone, Copy)]
struct WorldPosition {
    x: i32,
    y: i32,
    z: i32,
}

impl WorldPosition {
    fn to_voxel_position(self) -> VoxelPosition {
        let local_x = self.x.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let local_y = self.y as usize;
        let local_z = self.z.rem_euclid(CHUNK_WIDTH as i32) as usize;
        VoxelPosition { x: local_x, y: local_y, z: local_z }
    }

    fn to_chunk_position(self) -> ChunkPosition {
        let chunk_x = self.x.div_euclid(CHUNK_WIDTH as i32);
        let chunk_z = self.z.div_euclid(CHUNK_WIDTH as i32);
        ChunkPosition { x: chunk_x, z: chunk_z }
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
            world.chunks.insert(ChunkPosition { x, z }, chunk);
        }
    }

    // Now we can generate the meshes
    for (&chunk_pos, chunk) in &world.chunks {
        commands.spawn((
            Transform::from_translation(chunk_pos.origin()),
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
                let pos = VoxelPosition { x, y, z };
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

fn build_mesh(chunk: &Chunk, chunk_pos: ChunkPosition, world: &VoxelWorld, atlas: &TextureAtlasLayout) -> Mesh {
    let mut vertices = Vec::with_capacity(6 * 4); // 4 vertices per face
    let mut indices = Vec::with_capacity(6 * 6);  // 6 indices per face (2 triangles)
    let mut normals = Vec::with_capacity(6 * 4);  // 4 normals per face
    let mut uvs = Vec::with_capacity(6 * 4);      // 4 uvs per face

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
                    let VoxelPosition { x, y, z } = voxel_pos;

                    let (nx, ny, nz) = match direction {
                        Direction::Right => (0, y, z),
                        Direction::Left => (CHUNK_WIDTH - 1, y, z),
                        Direction::Front => (x, y, 0),
                        Direction::Back => (x, y, CHUNK_WIDTH - 1),
                        _ => unreachable!(),
                    };

                    let voxel_pos = VoxelPosition { x: nx, y: ny, z: nz };
                    let neighbor = neighbor_chunk[voxel_pos];
                    if !neighbor.is_transparent() {
                        continue;
                    }
                }
            }

            let face = Face { direction };
            let texture = voxel.texture(direction);
            let origin = vec3(voxel_pos.x as f32, voxel_pos.y as f32, voxel_pos.z as f32);

            let start = vertices.len() as u32;
            vertices.extend_from_slice(&face.vertices(origin));
            indices.extend_from_slice(&face.indices(start));
            normals.extend_from_slice(&face.normals());
            uvs.extend_from_slice(&atlas_uvs(atlas, texture));
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
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
                if world.is_solid(WorldPosition { x, y, z }) {
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
                if world.is_solid(WorldPosition { x, y, z }) {
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
                if world.is_solid(WorldPosition { x, y, z }) {
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
                if world.is_solid(WorldPosition { x, y, z }) {
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
                if world.is_solid(WorldPosition { x, y, z }) {
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
                if world.is_solid(WorldPosition { x, y, z }) {
                    // Put the player's back edge against the voxel
                    position.z = z as f32 + 1.0 + PLAYER_WIDTH_HALF;
                    player.velocity.z = 0.0;
                    return;
                }
            }
        }
    }
}
