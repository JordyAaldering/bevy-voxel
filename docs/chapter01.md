# Project setup

A new version of Bevy containing breaking changes to the API is released approximately once every 3 months, so follow the Bevy [quick start](https://bevy.org/learn/quick-start) to get a simple project up-and-running.

Bevy has a bunch of predefined [schedules](https://deepwiki.com/bevyengine/bevy/3.1-app-lifecycle-and-schedules).
The most common ones are `Startup`, `Update`, and `FixedUpdate`.
Many systems also have `Pre` and `Post` variants for additional granularity.
Libraries may also add their own schedules.
The order in which systems run can often be a source of bugs or unexpected behaviour, so it's important to think carefully about when systems that rely on each other exist in the same schedule.

There are other ways to [control the order of systems](https://bevy-cheatbook.github.io/programming/run-conditions.html), which we will discuss later.
This is often more convenient than adding that check as an early return in the system.

Ensure you have the `DefaultPlugins` plugin.
Add the `Startup` and `Update` systems to the app.
`Startup` runs only once at the start, we use it so spawn a cuboid, directional light, and a camera.
In `Update` we'll continuously handle user input.

# Our first 'voxel'

Spawn the camera.

```rust
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        todo!(),
        todo!(),
    ));
}
```

Give it a `Camera3d` and a `Transform`.
(Use autocompletion or documentation to find out possible ways of instantiating these components.)
Move the transform slightly back and up, so that the camera is not inside the cuboid.
We can easily make it look at the centre of the world with `Transform::looking_at`.

Also create a function to spawn a directional, global light.
Give it a `DirectionalLight`, and a `Transform`.
To determine the direction of the light, we use `Transform::from_rotation`, with as argument a quaternion, created with `Quat::from_euler`.

Finally, we create a function `spawn_voxel` that spawns a cuboid.
The cuboid has a mesh, and is given a material, so alongside `Commands` we also need to pass `Assets<Mesh>` and `Assets<StandardMaterial>` to the function.
Both need to be `ResMut<_>`, as the new mesh and material will be added to the respective asset storage.
The voxel of course needs a `Transform`.

To create the mesh, we create a new `Cuboid`, this creates a new asset which we add to the asset store with `Assets:add`.
The documentation of `Assets::add` states: _Adds the given asset and allocates a new strong Handle for it_.
In other words, it returns a handle to the newly created asset, which we use to create the `Mesh3d` component.
We similarly create a `MeshMaterial3d` component from a handle, by adding a `Color` to the material asset store.

Then, in `main`, add these three functions to the correct `system` schedule.

## Cursor locking

When looking around, we don't want the mouse cursor to leave the window.
For this, we create a `toggle_cursor` system that locks the cursor when the window is focused, and unlocks it when escape is pressed.
(You could also make a system for each, depending on your preference.)

The system needs three resources:
- `ButtonInput<KeyCode>` to check if escape is pressed,
- `ButtonInput<MouseButton>` to check if the left mouse button is pressed, and
- `Single<&mut CursorOptions, With<PrimaryWindow>>` to confine and hide the cursor.
  `Single` describes a resource that is guaranteed to have at most one entity with the given component, in this case `CursorOptions`.
  `CursorOptions` is mutable, so we can change the cursor grab mode and visibility.
  `With<PrimaryWindow>` ensures that we only get the cursor options for the primary window, in case there are multiple windows.

We use `ButtonInput::just_pressed` to check if a key (`KeyCode`) or `MouseButton` was just pressed.
If escape was pressed, we unlock the cursor and make it visible with `cursor.grab_mode` and `cursor.visible`, respectively.
Similarly, we lock the cursor and hide it when the left mouse button is pressed.
Because we added `With<PrimaryWindow>`, this only happens when the mouse is pressed inside the game window.

Don't forget to add the system.

## Looking around

We also want to be able to move the camera with the keyboard, and look around with the mouse.
Add two new systems: `player_look` and `player_move`.

To look around, we check how far the mouse moved in both the x and y direction, and rotate the camera accordingly.
These values are available in the `AccumulatedMouseMotion` resource.
To gain access to the camera we actually use `Single<&mut Transform, With<Camera3d>>` rather than `Single<&mut Camera3d>`, because we care about the camera's transform, not the camera itself.
Thus, `Single<&mut Transform, With<Camera3d>>` gives us the transform of the entity that also has a `Camera3d` component, of which there is only one.

There are some subtle details to consider when rotating the camera.
When the mouse moves right, this increases the x value of the mouse motion.
However, to look right, we rotate the camera **around** the y-axis: `camera.rotate_y(x)`.
We're still not done: passing `delta.x` rotates the camera in the wrong direction, so we negate it.
This occurs because [TODO].
Finally, we multiply the delta by some constant `MOUSE_SENSITIVITY` to slow down how fast the camera moves.

Do the same for the x-axis.
Start the game and look around a bit, you will notice that moving up and down causes the camera to roll.
How aggressive the camera rolls depends on where you positioned the camera.

If we check the documentation of `rotate_x`, it states: _Rotates this Transform around the X axis by angle (in radians). If this Transform has a parent, the axis is relative to the rotation of the parent_.
In this case, that means the rotation is relative to the world, rather than the camera's local rotation.
For `rotate_y` this is not a problem, because [TODO].

To make the x rotation relative to the camera's local rotation, we use `rotate_local_x` instead of `rotate_x`.

## Moving around

The movement system takes a keyboard and the camera, rather than the mouse and camera.
However, it needs one more resource: `Time`, which tells us how much time has passed since the last frame.
We need this to ensure the movement speed is independent of the frame rate.
If we would naively move the camera by a fixed amount each frame, the camera would move faster on high frame rates and slower on low frame rates.

The camera's position is then updated with: `camera.translation += MOVE_SPEED * time.delta_secs() * todo!()`.

How do we determine the direction in which the camera moves?
The simplest way is to define a mutable `Vec3`.
Then, for each key that is pressed (W, A, S, D, Space, Shift), we add the corresponding direction to the vector.
```rust
if keyboard.pressed(KeyCode::KeyW) {
    delta -= Vec3::Z;
}
```

If you do this for all directions, you will notice that moving forward does not move the camera in the direction it is looking, but always along the world's z-axis.
Similarly as in `player_look`, this is not a problem for the y-axis, because moving up and down is always along the world's y-axis.
Again, we need to move relative to the camera's rotation.
For moving forward, this means using `delta += *camera.forward()` instead.

If you try this out again you might notice another issue: moving diagonally is faster than moving in a single direction.
When moving diagonally, `delta` is `vec3(1, 0, 1)`, which has a length of `sqrt(2)`.
To ensure the length remains at most 1, we normalize the vector with `delta.normalize()`.
Ensure you only update the position if delta is non-zero.

> You don't need to do this with a bunch of if-statements. Alternatively, you can construct a `Vec3` directly from the keyboard input.

## Run conditions

We don't want the player to look and move then the game is not focused.

We could achieve this simply by defining a function:
```rust
fn is_focused(cursor: Single<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor.grab_mode == CursorGrabMode::Locked
}
```
And then adding a check at the start of the `player_look` and `player_move` functions.

However, Bevy has a more elegant solution: [run conditions](https://bevy-cheatbook.github.io/programming/run-conditions.html).
When we add the systems, we can add a `run_if` condition to them to ensure these systems only run when the window is focused.
```rust
(player_look, player_move).run_if(is_focused)
```
This keeps `player_look` and `player_move` simple.
