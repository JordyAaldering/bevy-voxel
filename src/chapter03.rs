use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    input::mouse::AccumulatedMouseMotion,
    mesh::{Indices, PrimitiveTopology},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::bevy_egui::prelude::*;
use bevy_inspector_egui::quick::{ResourceInspectorPlugin, WorldInspectorPlugin};

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

struct Voxel {
    x: usize,
    y: usize,
    z: usize,
}

impl Voxel {
    fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    fn origin(&self) -> Vec3 {
        vec3(self.x as f32, self.y as f32, self.z as f32)
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
            Self::Right  => vec3( 1.0, 0.0, 0.0),
            Self::Left   => vec3(-1.0, 0.0, 0.0),
            Self::Top    => vec3(0.0,  1.0, 0.0),
            Self::Bottom => vec3(0.0, -1.0, 0.0),
            Self::Front  => vec3(0.0, 0.0,  1.0),
            Self::Back   => vec3(0.0, 0.0, -1.0),
        }
    }
}

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            WorldInspectorPlugin::new(),
            ResourceInspectorPlugin::<CameraSettings>::default(),
        ))
        .init_resource::<CameraSettings>()
        .register_type::<CameraSettings>()
        .add_systems(Startup, (
            spawn_global_illumination,
            spawn_camera,
            spawn_voxel,
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

fn spawn_voxel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let voxel = Voxel::new(0, 0, 0);
    commands.spawn((
        Transform::from_translation(voxel.origin()),
        Mesh3d(meshes.add(generate_voxel_mesh(&voxel))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.7, 0.3))),
    ));
}

fn generate_voxel_mesh(voxel: &Voxel) -> Mesh {
    let mut vertices = Vec::with_capacity(6 * 4); // 4 vertices per face
    let mut indices = Vec::with_capacity(6 * 6);  // 6 indices per face (2 triangles)
    let mut normals = Vec::with_capacity(6 * 4);  // 4 normals per face
    let mut uvs = Vec::with_capacity(6 * 4);      // 4 uvs per face

    for face in [
        Direction::Right,
        Direction::Left,
        Direction::Top,
        Direction::Bottom,
        Direction::Front,
        Direction::Back,
    ] {
        let face = Face::new(voxel.origin(), face);

        let start = vertices.len() as u32;
        vertices.extend_from_slice(&face.vertices());
        indices.extend_from_slice(&face.indices(start));
        normals.extend_from_slice(&face.normals());
        uvs.extend_from_slice(&face.uvs());
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
    let mut delta = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        delta += *camera.forward();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        delta += *camera.back();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        delta += *camera.left();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        delta += *camera.right();
    }
    if keyboard.pressed(KeyCode::Space) {
        delta += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        delta -= Vec3::Y;
    }

    if delta != Vec3::ZERO {
        camera.translation += settings.move_speed * time.delta_secs() * delta.normalize();
    }
}
