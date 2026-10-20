mod coordinates;

use crate::coordinates::{ChunkCoord, VoxelCoord};

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
        matches!(self, VoxelType::Air)
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

#[derive(Default)]
struct MeshData {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
}

impl MeshData {
    fn add_face(&mut self, face: Face, origin: Vec3) {
        let start = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&face.vertices(origin));
        self.indices.extend_from_slice(&face.indices(start));
        self.normals.extend_from_slice(&face.normals());
        self.uvs.extend_from_slice(&face.uvs());
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
    fn get_chunk_neighbor(&self, pos: ChunkCoord, dx: i32, dz: i32) -> Option<&Chunk> {
        let neighbor_pos = ChunkCoord::new(pos.x + dx, pos.z + dz);
        self.chunks.get(&neighbor_pos)
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
        ))
        .init_resource::<VoxelWorld>()
        .init_resource::<CameraSettings>()
        .register_type::<CameraSettings>()
        .add_systems(EguiPrimaryContextPass, (
            chunk_debug_ui,
        ))
        .add_systems(Startup, (
            spawn_global_illumination,
            spawn_player,
            spawn_world,
        ))
        .add_systems(Update, (
            toggle_cursor,
            (player_look, player_move)
                .run_if(is_focused),
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
        Camera3d::default(),
        Transform::from_xyz(5.0, 6.0, 11.0)
            .looking_at(vec3(0.5, 0.5, 0.5), Vec3::Y),
    ));
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<VoxelWorld>,
) {
    let material = materials.add(
        Color::linear_rgb(0.2, 0.7, 0.3)
    );

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
            Mesh3d(meshes.add(build_mesh(chunk, chunk_pos, &world))),
            MeshMaterial3d(material.clone()),
        ));
    }
}

fn generate_chunk(chunk_x: i32, chunk_z: i32) -> Chunk {
    let mut chunk = Chunk { voxels: [VoxelType::Air; _] };

    for x in 0..CHUNK_WIDTH {
        for y in 0..(((chunk_x + chunk_z) as usize % CHUNK_HEIGHT) + 1) {
            for z in 0..CHUNK_WIDTH {
                let pos = VoxelCoord::new(x, y, z);
                chunk[pos] = VoxelType::Grass;
            }
        }
    }

    chunk
}

fn build_mesh(chunk: &Chunk, chunk_pos: ChunkCoord, world: &VoxelWorld) -> Mesh {
    let mut mesh_data = MeshData::default();

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

                    let voxel_pos = VoxelCoord { x: nx, y: ny, z: nz };
                    let neighbor = neighbor_chunk[voxel_pos];
                    if !neighbor.is_transparent() {
                        continue;
                    }
                }
            }

            let face = Face { direction };
            let origin = vec3(voxel_pos.x as f32, voxel_pos.y as f32, voxel_pos.z as f32);
            mesh_data.add_face(face, origin);
        }
    }

    mesh_data.build()
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
    let delta = vec3(
        f32::from(keyboard.pressed(KeyCode::KeyD)) - f32::from(keyboard.pressed(KeyCode::KeyA)),
        f32::from(keyboard.pressed(KeyCode::Space)) - f32::from(keyboard.pressed(KeyCode::ShiftLeft)),
        f32::from(keyboard.pressed(KeyCode::KeyW)) - f32::from(keyboard.pressed(KeyCode::KeyS)),
    );

    if delta != Vec3::ZERO {
        let direction = delta.x * camera.right() + delta.y * Vec3::Y + delta.z * camera.forward();
        camera.translation += settings.move_speed * time.delta_secs() * direction.normalize();
    }
}

fn chunk_debug_ui(mut contexts: EguiContexts, world: Res<VoxelWorld>) -> Result {
    egui::Window::new("Chunks").show(contexts.ctx_mut()?, |ui| {
        ui.label(format!("Loaded chunks: {}", world.chunks.len()));

        ui.separator();

        for (position, chunk) in &world.chunks {
            let solid_voxels = chunk.voxels.iter().filter(|&&v| v == VoxelType::Grass).count();

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
