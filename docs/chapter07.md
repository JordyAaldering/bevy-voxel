# Creating a player character

Adding a first person player, and especially the necessary physics, might seem daunting.
But it turns out that this is not actually as scary as it sounds in our simple voxel world.

First, we'll need some convenience functions.
For the physics, we'll definitely want to be able to look up arbitrary voxels in the world.
So add a `get_voxel(VoxelCoord) -> Option<VoxelType>` function to the world.
Use the x and z coordinate to determine the coordinate of the voxel, and then adjust the global voxel coordinates to chunk-local coordinates.
We don't place chunks vertically so y can remain unchanged, just check that it is in bounds.
Also add a function `is_solid`, which uses the `get_voxel` function to determine if a voxel is solid or not.

## Multiple game modes

For debugging, it is useful to retain the ability to fly around in spectator mode, so add a `GameMode` enum resource to switch between spectator and survival mode.
Like we did for toggling cursor grabbing, we add a system to switch the game mode when a certain key is pressed.

Also add two boolean function to check whether the current game mode is survival or spectator, which we can use to conditionally run the relevant movement and physics systems.

## Player instance

It's time to replace our mere camera with an actual player instance.
We define two new components, `Player` and `PlayerCamera`.
The player component we will later extend with the player's state.
The player camera component is just a marker component to identify the camera entity belonging to the player.

Modify the function that spawns the camera.

...

```rust
commands.spawn((
    Name::new("Player"),
    Camera3d::default(),
    Transform::from_xyz(5.0, 6.0, 11.0)
        .looking_at(vec3(0.5, 0.5, 0.5), Vec3::Y),
))
```

```rust
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
})
```

## Survival player

```rust
fn move_survival
```

Nested run conditions

## Physics

```rust
fn player_physics
```

For physics, we need to separate player input from the physics calculation.
We **must** run the input function every frame, because we might miss inputs.
But the physics simulation is typically run during the `FixedUpdate` schedule.

### Plugins

TODO: Implement physics as a plugin instead?
