use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};

const MOVE_SPEED: f32 = 1.0;
const MOUSE_SENSITIVITY: f32 = 0.001;

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

fn spawn_voxel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Transform::from_xyz(0.5, 0.5, 0.5),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.7, 0.3))),
    ));
}

fn toggle_cursor(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Release the cursor when escape is pressed
    if keyboard.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    // Recapture when a click occurs
    if cursor.grab_mode == CursorGrabMode::None
        && mouse.just_pressed(MouseButton::Left)
    {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn move_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
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
        camera.rotate_y(-delta.x * MOUSE_SENSITIVITY);

        camera.rotate_local_x(-delta.y * MOUSE_SENSITIVITY);
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
        camera.translation += MOVE_SPEED * time.delta_secs() * movement.normalize();
    }
}
