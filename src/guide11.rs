mod chunk;
mod chunk_mesh;
mod editor_tab_viewer;
mod face;
mod opensimplex2s;
mod telemetry;
mod texture_atlas;
mod voxel;
mod world;

use std::time::Duration;

use bevy::{camera::visibility::RenderLayers, diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin}, input::mouse::AccumulatedMouseMotion, prelude::*, time::common_conditions::on_timer, window::{CursorGrabMode, CursorOptions, PrimaryWindow}};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_egui::{EguiContext, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext}, prelude::*};

use crate::{chunk::*, editor_tab_viewer::EditorState, face::*, telemetry::{Telemetry, TelemetryPublisher}, voxel::*, world::*};

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

#[derive(Default, Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct DebugGizmos {
    #[inspector()]
    chunk_borders: bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            jump_strenght: 7.0,
            mouse_sensitivity: 0.00255,
        }
    }
}

#[derive(Debug)]
struct VoxelHit {
    position: WorldPosition,
    face: FaceDirection,
}

const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_WIDTH_HALF: f32 = PLAYER_WIDTH * 0.5;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_EYE_HEIGHT: f32 = 1.6;
const MAX_PITCH: f32 = 1.4;
const GRAVITY: f32 = 20.0;

const INTERACTION_DISTANCE: f32 = 5.0;

/// Prevents exact boundary contact from being treated as an overlap with the adjacent voxel.
const COLLISION_EPSILON: f32 = 1e-3;

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

fn main() {
    let (sender, receiver) = tokio::sync::watch::channel(Telemetry::default());

    // Run Tokio on its own thread, independently of the game
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Could not create Tokio runtime");
        runtime.block_on(telemetry::run_server(receiver));
    });

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_cursor_options: Some(CursorOptions {
                        grab_mode: CursorGrabMode::Locked,
                        visible: false,
                        ..default()
                    }),
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
            SystemInformationDiagnosticsPlugin::default(),
        ))
        .add_plugins(EguiPlugin::default())
        .add_plugins(DefaultInspectorConfigPlugin)
        .init_resource::<GameMode>()
        .init_resource::<VoxelWorld>()
        .init_resource::<CameraSettings>()
        .init_resource::<DebugGizmos>()
        .init_resource::<EditorState>()
        .register_type::<CameraSettings>()
        .register_type::<DebugGizmos>()
        .insert_resource(TelemetryPublisher(sender))
        .add_systems(EguiPrimaryContextPass, editor_tab_viewer::show_editor)
        .add_systems(PostUpdate, editor_tab_viewer::set_camera_viewport.after(editor_tab_viewer::show_editor))
        .add_systems(Startup, (
            spawn_global_illumination,
            spawn_player,
            setup_chunk_material,
        ))
        .add_systems(PreUpdate, (
            // These all depend on user input
            toggle_cursor,
            toggle_gamemode,
            player_look,
            player_move,
            break_place_block,
        ))
        .add_systems(Update, (
            draw_selected_voxel,
            spawn_chunks_around_player,
            retire_distant_chunks,
            remesh_chunks,
            telemetry::collect_telemetry.run_if(on_timer(Duration::from_millis(250)))
        ))
        .add_systems(PostUpdate, (
            // Do this at the end, because other systems might modify this chunk first
            despawn_retired_chunks,
        ))
        .run();
}

// fn debug_settings_ui(world: &mut World) {
//     let Ok(egui_context) = world
//         .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
//         .single(world)
//     else {
//         return;
//     };
//     let mut ctx = egui_context.clone();
//     egui::Window::new("Settings").show(ctx.get_mut(), |ui| {
//         ui.heading("Camera Settings");
//         ui.push_id("camera_settings", |ui| {
//             bevy_inspector::ui_for_resource::<CameraSettings>(world, ui);
//         });
//         ui.separator();
//         ui.heading("Debug Gizmos");
//         ui.push_id("debug_gizmos", |ui| {
//             bevy_inspector::ui_for_resource::<DebugGizmos>(world, ui);
//         });
//     });
// }

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

fn spawn_player(mut commands: Commands, mut egui_settings: ResMut<EguiGlobalSettings>) {
    // Give egui its own camera, so the game camera can be restricted to the editor's game view
    egui_settings.auto_create_primary_context = false;
    commands.spawn((
        PrimaryEguiContext,
        Camera2d,
        RenderLayers::none(),
        Camera { order: 1, ..default() },
    ));

    // In future we need to ensure to use some configuration to determine the x/y scaling (not relevant here because zero), the scaling, and the offset.
    let world_height = opensimplex2s::noise2d(0, 0.0, 0.0) * 10.0 + 20.0;
    commands
        .spawn((
            Name::new("Player"),
            Player {
                velocity: Vec3::ZERO,
                grounded: false,
            },
            Transform::from_xyz(1.5, world_height as f32 + 1.0, 1.5),
        ))
        .with_children(|player| {
            player.spawn((
                PlayerCamera,
                Camera3d::default(),
                Projection::from(PerspectiveProjection {
                    fov: 70f32.to_radians(),
                    ..default()
                }),
                Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 0.0),
            ));
        });
}

#[derive(Resource)]
struct ChunkMaterial(Handle<StandardMaterial>);

fn setup_chunk_material(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture: Handle<Image> = asset_server.load("blocks.png");
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..default()
    });
    commands.insert_resource(ChunkMaterial(material));
}

const CHUNK_LOAD_RADIUS: i32 = 1;

fn player_chunk_position(player: &Transform) -> ChunkPosition {
    world_to_chunk_position(WorldPosition::new(
        player.translation.x.floor() as i32,
        0,
        player.translation.z.floor() as i32,
    ))
}

/// Ensures the chunk the player is in, and the 8 chunks around it, exist.
/// Spawned chunks are only marked with `NeedsRemesh`; `remesh_chunks` meshes them.
fn spawn_chunks_around_player(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    material: Res<ChunkMaterial>,
    player: Single<&Transform, With<Player>>,
) {
    let center = player_chunk_position(&player);

    for dz in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
        for dx in -CHUNK_LOAD_RADIUS..=CHUNK_LOAD_RADIUS {
            let position = ChunkPosition::new(center.x + dx, center.z + dz);
            if world.is_loaded(position) {
                continue;
            }

            let entity = world.spawn_chunk(&mut commands, position);
            commands.entity(entity).insert(MeshMaterial3d(material.0.clone()));
        }
    }
}

/// Marks chunks outside the player's 3x3 area with `NeedsDespawn`, and unmarks
/// chunks that are back inside it. `despawn_retired_chunks` does the actual removal.
fn retire_distant_chunks(
    mut commands: Commands,
    player: Single<&Transform, With<Player>>,
    chunks: Query<(Entity, &Chunk, Has<NeedsDespawn>)>,
) {
    let center = player_chunk_position(&player);

    for (entity, chunk, retired) in &chunks {
        let in_range = (chunk.position.x - center.x).abs() <= CHUNK_LOAD_RADIUS
            && (chunk.position.z - center.z).abs() <= CHUNK_LOAD_RADIUS;

        if !in_range && !retired {
            commands.entity(entity).insert(NeedsDespawn);
        } else if in_range && retired {
            commands.entity(entity).remove::<NeedsDespawn>();
        }
    }
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

    // Horizontal movement rotates the entire player around the world Y axis
    player.rotate_y(-delta.x * settings.mouse_sensitivity);
    // Vertical movement rotates only the camera around its local X axis
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

    let min_x = (position.x - PLAYER_WIDTH_HALF).floor() as i32;
    let max_x = (position.x + PLAYER_WIDTH_HALF - COLLISION_EPSILON).floor() as i32;
    let min_z = (position.z - PLAYER_WIDTH_HALF).floor() as i32;
    let max_z = (position.z + PLAYER_WIDTH_HALF - COLLISION_EPSILON).floor() as i32;

    if velocity.y < 0.0 {
        // Falling
        let y = position.y.floor() as i32;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if world.is_solid(WorldPosition::new(x, y, z)) {
                    position.y = y as f32 + 1.0;
                    velocity.y = 0.0;
                    *grounded = true;
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
                if world.is_solid(WorldPosition::new(x, y, z)) {
                    position.y = y as f32 - PLAYER_HEIGHT;
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

    let min_y = position.y.floor() as i32;
    let max_y = (position.y + PLAYER_HEIGHT - COLLISION_EPSILON).floor() as i32;
    let min_z = (position.z - PLAYER_WIDTH_HALF).floor() as i32;
    let max_z = (position.z + PLAYER_WIDTH_HALF - COLLISION_EPSILON).floor() as i32;

    if velocity.x > 0.0 {
        let right = position.x + PLAYER_WIDTH_HALF;
        let x = (right - COLLISION_EPSILON).floor() as i32;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.is_solid(WorldPosition::new(x, y, z)) {
                    position.x = x as f32 - PLAYER_WIDTH_HALF;
                    velocity.x = 0.0;
                    return;
                }
            }
        }
    } else {
        let left = position.x - PLAYER_WIDTH_HALF;
        let x = left.floor() as i32;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.is_solid(WorldPosition::new(x, y, z)) {
                    position.x = x as f32 + 1.0 + PLAYER_WIDTH_HALF;
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

    let min_x = (position.x - PLAYER_WIDTH_HALF).floor() as i32;
    let max_x = (position.x + PLAYER_WIDTH_HALF - COLLISION_EPSILON).floor() as i32;
    let min_y = position.y.floor() as i32;
    let max_y = (position.y + PLAYER_HEIGHT - COLLISION_EPSILON).floor() as i32;

    if velocity.z > 0.0 {
        let front = position.z + PLAYER_WIDTH_HALF;
        let z = (front - COLLISION_EPSILON).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if world.is_solid(WorldPosition::new(x, y, z)) {
                    // Put the player's front edge against the voxel
                    position.z = z as f32 - PLAYER_WIDTH_HALF;
                    velocity.z = 0.0;
                    return;
                }
            }
        }
    } else {
        let back = position.z - PLAYER_WIDTH_HALF;
        let z = back.floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if world.is_solid(WorldPosition::new(x, y, z)) {
                    // Put the player's back edge against the voxel
                    position.z = z as f32 + 1.0 + PLAYER_WIDTH_HALF;
                    velocity.z = 0.0;
                    return;
                }
            }
        }
    }
}

fn draw_selected_voxel(
    mut gizmos: Gizmos,
    world: Res<VoxelWorld>,
    camera: Single<&GlobalTransform, With<PlayerCamera>>,
) {
    let Some(hit) = find_voxel(
        &world,
        camera.translation(),
        *camera.forward(),
    ) else {
        return;
    };

    let origin = hit.position.origin();
    let face = Face::new(origin, hit.face);
    let vertices = face.vertices();

    gizmos.line(vertices[0], vertices[1], Color::WHITE);
    gizmos.line(vertices[1], vertices[3], Color::WHITE);
    gizmos.line(vertices[3], vertices[2], Color::WHITE);
    gizmos.line(vertices[2], vertices[0], Color::WHITE);
}

fn break_place_block(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut world: ResMut<VoxelWorld>,
    camera: Single<&GlobalTransform, With<PlayerCamera>>,
) {
    let do_break = mouse.just_pressed(MouseButton::Left);
    let do_place = mouse.just_pressed(MouseButton::Right);
    if !(do_break || do_place) {
        return;
    }

    let origin = camera.translation();
    let direction = camera.forward();

    if let Some(VoxelHit { position, face }) = find_voxel(&world, origin, *direction) {
        info!("Hit {:?} at {:?} face", position, face);
        if do_break {
            world.set_voxel(&mut commands, position, VoxelType::Air);
        } else /* if do_place */ {
            let neighbor = position.neighbor(face);
            world.set_voxel(&mut commands, neighbor, VoxelType::Stone);
        }
    }
}

fn find_voxel(world: &VoxelWorld, origin: Vec3, direction: Vec3) -> Option<VoxelHit> {
    let direction = direction.normalize();

    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    let step_x = direction.x.signum() as i32;
    let step_y = direction.y.signum() as i32;
    let step_z = direction.z.signum() as i32;

    let delta_x = if direction.x != 0.0 {
        (1.0 / direction.x).abs()
    } else {
        f32::INFINITY
    };

    let delta_y = if direction.y != 0.0 {
        (1.0 / direction.y).abs()
    } else {
        f32::INFINITY
    };

    let delta_z = if direction.z != 0.0 {
        (1.0 / direction.z).abs()
    } else {
        f32::INFINITY
    };

    let mut max_x = initial_axis_distance(origin.x, direction.x);
    let mut max_y = initial_axis_distance(origin.y, direction.y);
    let mut max_z = initial_axis_distance(origin.z, direction.z);

    loop {
        let (distance, face) = if max_x <= max_y && max_x <= max_z {
            x += step_x;

            let distance = max_x;
            max_x += delta_x;

            let face = if step_x > 0 {
                FaceDirection::Left
            } else {
                FaceDirection::Right
            };

            (distance, face)
        } else if max_y <= max_z {
            y += step_y;

            let distance = max_y;
            max_y += delta_y;

            let face = if step_y > 0 {
                FaceDirection::Bottom
            } else {
                FaceDirection::Top
            };

            (distance, face)
        } else {
            z += step_z;

            let distance = max_z;
            max_z += delta_z;

            let face = if step_z > 0 {
                FaceDirection::Back
            } else {
                FaceDirection::Front
            };

            (distance, face)
        };

        if distance > INTERACTION_DISTANCE {
            return None;
        }

        let position = WorldPosition::new(x, y, z);

        if world.is_solid(position) {
            return Some(VoxelHit {
                position,
                face,
            });
        }
    }
}

/// Returns the distance until the ray crosses the next voxel boundary on this axis.
/// This is needed to determine which neighboring voxel to visit next.
fn initial_axis_distance(origin: f32, direction: f32) -> f32 {
    if direction > 0.0 {
        (origin.ceil() - origin) / direction
    } else if direction < 0.0 {
        (origin.floor() - origin) / direction
    } else {
        f32::INFINITY
    }
}
