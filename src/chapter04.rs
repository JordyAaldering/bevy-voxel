use std::ops::{Index, IndexMut};

use better_default::Default;
use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    input::mouse::AccumulatedMouseMotion,
    mesh::{Indices, PrimitiveTopology},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::bevy_egui::prelude::*;
use bevy_inspector_egui::bevy_egui::input::EguiWantsInput;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const CHUNK_WIDTH: usize = 32;
const CHUNK_HEIGHT: usize = 64;

#[derive(Default, Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct CameraSettings {
    #[default(5.0)]
    #[inspector(min = 0.1, max = 20.0)]
    move_speed: f32,
    #[default(0.0025)]
    #[inspector(min = 0.0001, max = 0.01)]
    mouse_sensitivity: f32,
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

/// Chunk-local voxel position, with x and z in [0, CHUNK_WIDTH) and y in [0, CHUNK_HEIGHT).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct VoxelPosition {
    x: usize,
    y: usize,
    z: usize,
}

impl VoxelPosition {
    fn new(x: usize, y: usize, z: usize) -> Self {
        debug_assert!(x < CHUNK_WIDTH);
        debug_assert!(y < CHUNK_HEIGHT);
        debug_assert!(z < CHUNK_WIDTH);
        Self { x, y, z }
    }

    fn origin(&self) -> Vec3 {
        vec3(self.x as f32, self.y as f32, self.z as f32)
    }

    fn to_index(self) -> usize {
        self.x + CHUNK_WIDTH * (self.y + CHUNK_HEIGHT * self.z)
    }
}

struct Face {
    origin: Vec3,
    direction: Direction,
}

impl Face {
    fn new(origin: Vec3, direction: Direction) -> Self {
        Self { origin, direction }
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
    fn vertices(&self) -> [Vec3; 4] {
        let o = self.origin;

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

    /// Returns the UV coordinates of the 4 corners of the quad.
    fn uvs(&self) -> [Vec2; 4] {
        [
            vec2(0.0, 0.0), // Bottom left
            vec2(1.0, 0.0), // Bottom right
            vec2(0.0, 1.0), // Top left
            vec2(1.0, 1.0), // Top right
        ]
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
    fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    fn uvs(&self, tile: TextureAtlasPosition) -> [Vec2; 4] {
        let tile_width = 1.0 / self.width as f32;
        let tile_height = 1.0 / self.height as f32;
        let x = tile.x as f32 * tile_width;
        let y = tile.y as f32 * tile_height;
        // Note that the Y axis is flipped in UV coordinates, so we need to flip the Y coordinate of the tile position
        [
            vec2(x, y + tile_height),              // Bottom left
            vec2(x + tile_width, y + tile_height), // Bottom right
            vec2(x, y),                            // Top left
            vec2(x + tile_width, y),               // Top right
        ]
    }
}

impl TextureAtlasPosition {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

struct Chunk {
    voxels: [VoxelType; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH],
}

#[derive(Default)]
struct MeshData {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
}

impl Chunk {
    fn new() -> Self {
        Self {
            voxels: [VoxelType::Air; _],
        }
    }

    fn get_neighbor_voxel(&self, pos: VoxelPosition, direction: Direction) -> Option<VoxelType> {
        let VoxelPosition { x, y, z } = pos;

        let (x, y, z) = match direction {
            Direction::Right => (x + 1, y, z),
            Direction::Left => (x.wrapping_sub(1), y, z),
            Direction::Top => (x, y + 1, z),
            Direction::Bottom => (x, y.wrapping_sub(1), z),
            Direction::Front => (x, y, z + 1),
            Direction::Back => (x, y, z.wrapping_sub(1)),
        };

        if x < CHUNK_WIDTH && y < CHUNK_HEIGHT && z < CHUNK_WIDTH {
            let pos = VoxelPosition { x, y, z };
            let idx = pos.to_index();
            Some(self.voxels[idx])
        } else {
            None
        }
    }
}

impl Index<VoxelPosition> for Chunk {
    type Output = VoxelType;

    fn index(&self, pos: VoxelPosition) -> &Self::Output {
        &self.voxels[pos.to_index()]
    }
}

impl IndexMut<VoxelPosition> for Chunk {
    fn index_mut(&mut self, pos: VoxelPosition) -> &mut Self::Output {
        &mut self.voxels[pos.to_index()]
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ))
        .init_resource::<CameraSettings>()
        .register_type::<CameraSettings>()
        .add_systems(Startup, (
            spawn_global_illumination,
            spawn_camera,
            spawn_chunk,
        ))
        .add_systems(Update, (
            toggle_cursor,
            (player_look, player_move).run_if(is_focused),
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        Camera3d::default(),
        Transform::from_xyz(1.5, 2.0, 4.0)
            .looking_at(vec3(0.5, 0.5, 0.5), Vec3::Y),
    ));
}

fn spawn_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut chunk = Chunk::new();

    for x in 0..CHUNK_WIDTH {
        for y in 0..(CHUNK_HEIGHT / 2) {
            for z in 0..CHUNK_WIDTH {
                let pos = VoxelPosition::new(x, y, z);
                chunk[pos] = VoxelType::Grass;
            }
        }
    }

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(build_mesh(&chunk))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.7, 0.3))),
    ));
}

fn build_mesh(chunk: &Chunk) -> Mesh {
    let mut mesh = MeshData::default();

    for z in 0..CHUNK_WIDTH {
    for y in 0..CHUNK_HEIGHT {
    for x in 0..CHUNK_WIDTH {
        let pos = VoxelPosition { x, y, z };
        let voxel = chunk[pos];

        if voxel == VoxelType::Air {
            continue;
        }

        let origin = pos.origin();

        for face in [
            Direction::Right,
            Direction::Left,
            Direction::Top,
            Direction::Bottom,
            Direction::Front,
            Direction::Back,
        ] {
            if let Some(neighbor) = chunk.get_neighbor_voxel(pos, face) {
                if !neighbor.is_transparent() {
                    continue;
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
    }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh.uvs)
        .with_inserted_indices(Indices::U32(mesh.indices))
}

fn is_focused(cursor: Single<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor.grab_mode == CursorGrabMode::Locked
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

fn player_look(
    settings: Res<CameraSettings>,
    mouse: Res<AccumulatedMouseMotion>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    let delta = mouse.delta;

    if delta != Vec2::ZERO {
        camera.rotate_y(-delta.x * settings.mouse_sensitivity);
        camera.rotate_local_x(-delta.y * settings.mouse_sensitivity);
    }
}

fn player_move(
    time: Res<Time>,
    settings: Res<CameraSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    let delta = Vec3::new(
        f32::from(keyboard.pressed(KeyCode::KeyD)) - f32::from(keyboard.pressed(KeyCode::KeyA)),
        f32::from(keyboard.pressed(KeyCode::Space)) - f32::from(keyboard.pressed(KeyCode::ShiftLeft)),
        f32::from(keyboard.pressed(KeyCode::KeyW)) - f32::from(keyboard.pressed(KeyCode::KeyS)),
    );

    if delta != Vec3::ZERO {
        let direction = delta.x * camera.right() + delta.y * Vec3::Y + delta.z * camera.forward();
        camera.translation += settings.move_speed * time.delta_secs() * direction.normalize();
    }
}
