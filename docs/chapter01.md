# The first voxel

Before we can start working on voxels, we need a basic Bevy project.
Bevy releases breaking API changes approximately every three months, so follow the [quick start](https://bevy.org/learn/quick-start) to get a project up and running.

Bevy uses _schedules_ to determine when systems run.
The most common ones are `Startup`, `Update`, and `FixedUpdate`.
`Startup` runs once when the app starts, while `Update` runs every frame.
`FixedUpdate` runs at a fixed time interval, which is useful for things like physics.
Bevy also provides `Pre` and `Post` schedules for additional control, and plugins can introduce their own schedules.

The order in which systems run can be a source of bugs, especially when one system depends on another.
For now, we'll use `Startup` to spawn our scene and `Update` to handle player input.
We'll look at more ways to control system execution later.

Make sure your app includes `DefaultPlugins`, then add the systems described below to their respective schedules.

## Spawning the scene

Let's start with something simple: a pre-defined cuboid, a light, and a camera.
We'll use a cuboid as our first voxel, even though we aren't building an actual voxel system just yet.

First, create a system to spawn the camera:

```rust
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        todo!(),
        todo!(),
    ));
}
```

Give the entity a `Camera3d` and a `Transform`, and optionally a `Name` for debugging.
If you're unsure how to construct these components, use your IDE's autocompletion or consult the documentation.
Move the camera slightly up and back so it isn't inside the cuboid, and use `Transform::looking_at` to point it towards the centre of the world.

Next, create a system to spawn a directional light.
Like the camera, it needs a `Transform`, alongside a `DirectionalLight`.
We can use `Transform::from_rotation` to set its direction, passing in a quaternion created with `Quat::from_euler`.

Finally, create a `spawn_voxel` system.
This system spawns a cuboid with a mesh, a material, and a `Transform`.

To create the mesh and material, we need access to Bevy's asset storage.
Add `ResMut<Assets<Mesh>>` and `ResMut<Assets<StandardMaterial>>` as system parameters.
`ResMut` gives us mutable access to these resources, allowing us to add new assets.

We create a mesh from a `Cuboid` and add it to the mesh asset storage using `Assets::add`.
As the [documentation](https://docs.rs/bevy/latest/bevy/asset/struct.Assets.html#method.add) explains, this returns a strong handle to the newly added asset.
We use that handle to construct the `Mesh3d` component.

The material works in much the same way.
Create a material using a `Color`, add it to the material asset storage, and use the returned handle to construct a `MeshMaterial3d` component.

With all three systems implemented, add them to the appropriate schedules in `main`.
Run the game and you should see a lit cuboid.

## Cursor locking

To look around with the mouse, we don't want the cursor to leave the game window.
Let's implement a system that locks and hides the cursor when the left mouse button is pressed, and unlocks and shows it when Escape is pressed.

We'll call this system `toggle_cursor`.
It needs three parameters:
- `ButtonInput<KeyCode>` to check whether Escape is pressed.
- `ButtonInput<MouseButton>` to check whether the left mouse button is pressed.
- `Single<&mut CursorOptions, With<PrimaryWindow>>` to access the cursor settings of the primary window.

`Single` lets us access a component on an entity when we expect exactly one matching entity.
Here, `With<PrimaryWindow>` ensures we get the cursor settings for the primary window, even if the app has multiple windows.
Since `CursorOptions` is mutable, we can change its grab mode and visibility.

Use `ButtonInput::just_pressed` to check whether either input was pressed during the current frame.
When Escape is pressed, set `cursor.grab_mode` to unlock the cursor and set `cursor.visible` to `true`.
When the left mouse button is pressed, do the opposite.

Add the system to `Update` and try it out.
You should now be able to lock and unlock the cursor while the game is running.

## Looking around

Now that we can capture the mouse, let's use it to look around.
We'll add two systems: `player_look` and `player_move`.
We'll start with the former.

Bevy provides an `AccumulatedMouseMotion` resource containing the mouse movement accumulated during the current frame.
We can use its horizontal and vertical deltas to rotate the camera.

To access the camera, we use `Single<&mut Transform, With<Camera3d>>`, rather than `Single<&mut Camera3d>`.
We want to change the camera's position and rotation, which are stored in its `Transform`, not the `Camera3d` component itself.

Start by rotating the camera around the Y-axis using `rotate_y`.
When the mouse moves to the right, its horizontal delta is positive, but we need to negate that delta to make the camera look to the right.
This is because a camera looks along its local negative Z-axis, so the positive rotation around Y turns its forward direction towards the left.

We also multiply the delta by a `MOUSE_SENSITIVITY` constant to control how quickly the camera rotates.

Try implementing this and see how it feels.
Adjust the sensitivity until the movement is comfortable.

Next, apply the vertical mouse delta using `rotate_x`.
If you try this, you'll notice that looking up and down can cause the camera to roll.
How much it rolls depends on the camera's current orientation.

The problem is that `rotate_x` rotates around the X-axis relative to the parent, or the world if there is no parent.
We want to pitch the camera around its own local X-axis instead.
For horizontal rotation, the world Y-axis is appropriate because we want to turn around the world's vertical axis.
For vertical rotation, however, we want to rotate relative to the camera's orientation.

Use `rotate_local_x` instead of `rotate_x`.
This makes the vertical rotation relative to the camera's local X-axis, preventing the unwanted rolling.

## Moving around

Let's implement `player_move` next.
Unlike `player_look`, this system needs keyboard input and the camera's transform.
It also needs a `Time` resource.

Why? We want movement speed to be independent of the frame rate.
If we move the camera by a fixed amount every frame, it will move faster on machines that render more frames per second.
Multiplying the movement by `time.delta_secs()` ensures that movement is proportional to the time elapsed since the previous frame.

The camera's position can then be updated like this:

```rust
camera.translation += MOVE_SPEED * time.delta_secs() * todo!();
```

The missing piece is the movement direction.
One approach is to start with a mutable `Vec3` and add a direction for every key that is pressed.
For example:

```rust
if keyboard.pressed(KeyCode::KeyW) {
    delta -= Vec3::Z;
}
```

Repeat this for the other movement keys: A, S, D, Space, and Shift.

Try it out.
You'll notice that moving forward always moves the camera along the world's Z-axis, regardless of which way you're looking.
That's not what we want.
As with camera rotation, we need to distinguish between world-space and local-space directions.

Fortunately, Bevy provides methods such as `camera.forward()` that return the camera's direction in world space.
Instead of adding `-Vec3::Z` when W is pressed, add `camera.forward()`.
Apply the same idea to the other horizontal directions using the camera's right vector.

For vertical movement, we can continue to use the world's Y-axis.
Pressing Space should move us up, and Shift should move us down, regardless of the camera's orientation.

There's one more issue.
If you move diagonally, you'll notice that the camera moves faster than when moving in a single direction.
This happens because combining two perpendicular directions produces a vector with a length of \(\sqrt{2}\), rather than \(1\).

We can fix this by normalizing the movement vector with `delta.normalize()`.
This gives us a vector of length \(1\), preserving the movement speed regardless of direction.
However, only normalize the vector if it isn't zero; otherwise, you'll end up with an invalid vector.

You don't need to use a bunch of `if` statements to construct the movement vector.
You can also derive a `Vec3` directly from the keyboard input.
The important part is understanding how the movement direction is determined and why it needs to be normalized.

## Run conditions

At this point, we can look around and move through the scene.
However, we shouldn't do either when the game window isn't focused.

One way to handle this is to check at the start of both `player_look` and `player_move` whether the cursor is locked:

```rust
fn is_focused(cursor: Single<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor.grab_mode == CursorGrabMode::Locked
}
```

We could call this function from both systems and return early if it returns `false`.
However, Bevy provides a cleaner solution: [run conditions](https://bevy-cheatbook.github.io/programming/run-conditions.html).

When adding the systems, we can attach a condition that determines whether they should run:

```rust
(player_look, player_move).run_if(is_focused)
```

Now Bevy only runs these systems when the condition is satisfied, keeping the movement and camera logic separate from the question of when that logic should execute.

Run conditions are useful for much more than cursor locking.
They let us control when systems execute without filling those systems with checks.
We'll use them again as the project grows.
