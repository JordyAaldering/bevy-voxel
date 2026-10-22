# The first voxel

Before we can start working on voxels, we need a basic Bevy project.
Bevy releases breaking API changes approximately every three months, so follow the [quick start](https://bevy.org/learn/quick-start) to get a project up and running.
Make sure you correctly configure your Cargo configuration, you don't want to spend hours waiting for compilation.

Bevy uses _schedules_ to determine when systems run.
The most common ones are `Startup`, `Update`, and `FixedUpdate`.

- `Startup` runs once when the app starts,
- `Update` runs every frame, and
- `FixedUpdate` runs at a fixed time interval, which is useful for things like physics.

Bevy also provides `Pre` and `Post` schedules for additional control, and plugins can also introduce their own schedules.
The order in which systems run can be a source of bugs, especially when systems depend on one another.
When your project gets more complicated, think carefully about the ordering of systems.

## Spawning the scene

We can't do anything before we can look and move around in our world.
We start with a simple scene, containing a camera that we can move around, a pre-defined Bevy voxel, and a global light so we can actually see something.

The modern approach for spawning a scene in Bevy is with the [Bevy Scene Notation](https://taintedcoders.com/bevy/bsn).
The scene we'll be creating is very simple, so you can also just create a system for spawning each entity with `Commands::spawn`.
Whichever you choose, create a function for spawning the camera, the directional light, and the voxel.

### The camera

The camera entity needs a `Camera3d` and `Transform` component.
Optionally, you can add a `Name` component for debugging.
Use the default camera component, but move the transform slightly up, back, and to the right so it isn't inside the voxel.
If you're unsure how to construct these components, use your IDE's autocompletion or consult the documentation.
To make the camera look at the voxel we can use `Transform::looking_at`, no rotational math required.

### Global lighting

The directional light needs a `DirectionalLight` and `Transform` component.
We use mostly the default directional light settings, but enable `shadow_maps_enabled` so we can see shadows.
Shadows are expensive, so it is best to aggressively limit the number of lights that cast shadows.
The position of the transform doesn't matter, but the rotation does.
To create a transform with a specific rotation, we can use `Transform::from_rotation`.
Rotations are represented as quaternions: 4-dimensional vectors that can represent any rotation in 3D space.
Quaternions are hard to understand, but fortunately we can construct one from just the x, y, and z angles using `Quat::from_euler`.

### The voxel

We won't yet be creating our own voxel.
First, we'll use a pre-defined Bevy voxel: a `Cuboid`.
It needs two components: a mesh, and a material.
The mesh defines the shape of the voxel, and the material defines how the surface of that shape looks.
The mesh component, `Mesh3d`, has a single field `Handle<Mesh>`, which is a reference to the actual mesh data.
It works like this because meshes and materials may be shared between entities.
The mesh data itself lives in _asset stores_, which we access by adding `mut meshes: ResMut<Assets<Mesh>>` as an argument.
We then add a `Cuboid` to the mesh asset store with `meshes.add`.
Adding a `MeshMaterial3d` works the same way, but we add a `StandardMaterial` to the material asset store instead.
The material itself is just a colour, created from RGB values with `Color::linear_rgb`.
As most things, the entity also needs a `Transform`.

With all three systems in place, add them to the `Startup` schedule and run the app.
You should see a voxel in the middle of the screen.

## Cursor locking

When looking around, we don't want the cursor to leave the game window.
Before we implement looking, we need to lock the cursor to the window when we click inside it.
When the escape key is pressed, we want to unlock the cursor and show it again.

This system requires three arguments:

- `ButtonInput<KeyCode>` to check whether a certain key is pressed,
- `ButtonInput<MouseButton>` to check whether a mouse button is pressed, and
- `Single<&mut CursorOptions, With<PrimaryWindow>>` to access the cursor settings of the primary window.

`Single` lets us access a component on an entity when we expect exactly one matching entity in the entire game.
We use `With<PrimaryWindow>` to ensure that we only get the cursor settings for the primary window, even if the app has multiple windows.
To change the cursor's grab mode and visibility, we make the cursor options mutable.

If the user `just_pressed` the escape key, we unlock the cursor and make it visible.
This is done by setting the cursor's `grab_mode` to `CursorGrabMode::None` and its `visible` field to `true`.
We do the inverse when the left mouse button is pressed: we lock the cursor (`CursorGrabMode::Locked`) and hide it.
We don't need to check whether the mouse is pressed inside the window, because any inputs outside the game view are not captured by Bevy anyway.

I wasn't sure whether there would be a specific schedule for handling user input, so I googled (DuckDuckGo'd?) it.
Search assist confidently suggested: _You should use the `FixedUpdate` schedule for capturing and handling user input [...]_.
This is **wrong**!
The fixed-update schedule runs at a fixed interval, and is used for things like physics.
Because it runs less frequently than the update schedule, it can miss input events.
This goes to show that you shouldn't blindly trust AI-generated answers.

In any case, there is no special schedule for input handling, so we can just add the system to the `Update` schedule.
`PreUpdate` can also make sense to separate collecting input from processing it.
You could even make your own schedule that sits in between those two to further separate concerns and make code more self-documenting.
Pick whichever you prefer.

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
This gives us a vector of length 1, preserving the movement speed regardless of direction.
However, only normalize the vector if it isn't zero; otherwise, you'll end up with an invalid vector.

You don't need to use a bunch of `if` statements to construct the movement vector.
You can also derive a `Vec3` directly from the keyboard input.
The important part is understanding how the movement direction is determined and why it needs to be normalized.

## Run conditions

At this point, we can look around and move through the scene.
However, we shouldn't do either when the game window isn't focused.

One way to handle this is to check at the top of relevant systems whether the cursor is locked, returning early if it isn't.

```rust
fn is_focused(cursor: Single<&CursorOptions, With<PrimaryWindow>>) -> bool {
    cursor.grab_mode == CursorGrabMode::Locked
}
```

Bevy provides a cleaner solution: [run conditions](https://bevy-cheatbook.github.io/programming/run-conditions.html).
When adding the systems, we can attach a condition that determines whether they should run using `(a, b).run_if(is_focused)`.
Bevy then only runs systems _a_ and _b_ when the condition is satisfied, keeping those systems' logic separate from whether that system should execute.
