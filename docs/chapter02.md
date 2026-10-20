# Inspecting settings with egui

In the previous chapter, we used constants to control the camera's movement speed and mouse sensitivity.
Every time we wanted to tune these values, we had to change the code and recompile.
It would be much more convenient to adjust them while the game is running.

We can do this with [egui](https://github.com/taboky-dev/bevy-inspector-egui), which lets us inspect and edit values through a GUI.
Add `bevy-inspector-egui` as a dependency and make sure the `EguiPlugin` is added to the app.

## Replacing constants with a resource

Constants aren't meant to change at runtime, so let's replace them with a `CameraSettings` struct containing the movement speed and mouse sensitivity.

Since these settings need to be accessible from our systems, we'll make the struct a Bevy resource by deriving `Resource`.
We also need to give it default values and register it with the app using `init_resource::<CameraSettings>()`.

You can implement `Default` manually or derive it using the `better_default` crate.
I prefer the latter because it keeps the default values close to the struct definition.
[Similar functionality](https://github.com/rust-lang/rust/issues/132162) is also available in nightly Rust and may be available in stable Rust by the time you read this.

Now, update `player_move` and `player_look` to use the resource instead of the constants.

## Inspecting resources

The `WorldInspectorPlugin` provides an inspector in the game window that lets us browse entities, components, resources, and assets.
Add it to the app and look for `CameraSettings`.

You'll notice it isn't there.
The inspector needs to know how to inspect a type at runtime, and for that it uses Bevy's reflection system.

Reflection provides runtime type information that allows tools to inspect and manipulate values without needing to know their concrete types at compile time.
To make `CameraSettings` available to the inspector, derive `Reflect` and add `#[reflect(Resource)]` to the struct.
Then register the type with the app using `register_type::<CameraSettings>()`.

Run the game again.
You should now be able to find `CameraSettings` in the inspector and change its values while the game is running.

These changes are only temporary.
When you close the game, the values are lost, so remember to update the default values in your struct if you want to keep your adjustments.

Currently, the inspector also allows you to enter arbitrary values, which can cause problems.
For example, a negative movement speed reverses the direction of movement.
We can constrain the values using `InspectorOptions`.
Add it to the relevant derives and reflection attributes, then specify bounds for each field:

```rust
#[inspector(min = 0.1, max = 20.0)]
```

Adjust the bounds to suit each setting.
This prevents the inspector from accepting values outside the specified range.

The world inspector is useful, but it can be overwhelming when all we want to change is the camera settings.
Instead, add `ResourceInspectorPlugin::<CameraSettings>::default()` to the app.
This gives us a dedicated window for editing that resource.

## Fixing cursor grabbing

There's one problem left.
If you try clicking the inspector, the game also grabs the cursor.
We need to prevent `toggle_cursor` from capturing the mouse when the click is intended for the GUI.

A quick search leads us to [`EguiWantsInput`](https://docs.rs/bevy_egui/latest/bevy_egui/input/struct.EguiWantsInput.html).
Read its documentation to understand how egui indicates that it wants keyboard or pointer input, then use it in `toggle_cursor` to avoid grabbing the cursor when the GUI is handling the click.

This is a useful example of why it's worth looking up the relevant documentation instead of guessing at an implementation.
The GUI library already tracks the information we need; we just have to find out how to use it.

With that fixed, we can tune the camera settings without recompiling or accidentally losing control of the GUI.
We'll use the same approach whenever it becomes useful to inspect or adjust values while developing the game.
