use bevy::prelude::*;
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

const MOVE_SPEED: f32 = 1.0;
const MOUSE_SENSITIVITY: f32 = 0.001;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
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
    commands.spawn((
        Transform::from_xyz(0.5, 0.5, 0.5),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.2, 0.7, 0.3))),
    ));
}

fn is_focused(cursor: Single<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor.grab_mode == CursorGrabMode::Locked
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

fn player_look(
    mouse: Res<AccumulatedMouseMotion>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    let delta = mouse.delta;

    if delta != Vec2::ZERO {
        camera.rotate_y(-delta.x * MOUSE_SENSITIVITY);
        camera.rotate_local_x(-delta.y * MOUSE_SENSITIVITY);
    }
}

fn player_move(
    time: Res<Time>,
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
        camera.translation += MOVE_SPEED * time.delta_secs() * delta.normalize();
    }
}
