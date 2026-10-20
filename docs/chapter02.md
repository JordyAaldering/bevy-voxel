# Inspecting settings with egui

When tuning the move speed and mouse sensitivity in the previous chapter, you probably changed the hardcoded constants and recompiled to tune the values.
That is incredibly inconvenient, and it would be nice to be able to change these values whilst the game is running.
We can do this by adding a simple UI in the game view using [egui](https://github.com/taboky-dev/bevy-inspector-egui.git)!
Add a `bevy-inspector-egui` dependency.
We are required to add the `EguiPlugin` plugin.

## Replacing constants with a resource

By definition, we cannot change constants.
Instead, replace them with a single `CameraSettings` struct.
It is a Bevy resource, so derive `Resource` for it, and add `init_resource::<CameraSettings>()` in main to make it available to the systems.
This also requires the settings to implement `Default`.

You can either implement `Default` manually, or derive it with `#[derive(Default)]`.
For the second option, you need to add the `better_default` dependency.
I prefer this approach because it keeps the default values close to the struct definition.

> [The same functionality](https://github.com/rust-lang/rust/issues/132162) is also available in nightly Rust, and might be available in stable Rust at the time of reading.

## Adding an inspector in the game view

We can add the `WorldInspectorPlugin` plugin to the app, which will add an inspector in the game view containing all entities, resources, and assets.
However, if you look through the resources you will not find the `CameraSettings` resource.

This is because the resource does not implement `Reflect`.
[Reflection is a way to inspect types at runtime, and is required for the inspector to work. Accurate?]
Derive `Reflect`, and add `#[reflect(Resource)]` to the struct.
Finally, you need to register the type to the app: after `init_resource::<CameraSettings>()`, add `register_type::<CameraSettings>()`.
Now, if you run the game, you will find the `CameraSettings` resource in the inspector, and you can change the values in real time!
These values will be lost after the game is closed, so note down the values you want to keep and adjust the `CameraSettings` default values when you are done.

Currently, you can enter any value in the inspector, which can lead to unexpected behaviour, like a negative move speed.
By adding `InspectorOptions` to both `#[derive()]` and `#[reflect()]`, we can add a min and max value to the fields, which will be enforced in the inspector.
Then, for fields we can define bounds like `#[inspector(min = 0.1, max = 20.0)]`.

Looking through the entire `WorldInspectorPlugin` is a bit overwhelming, and we can add a dedicated inspector for the `CameraSettings` resource.
This is done by adding the `ResourceInspectorPlugin::<CameraSettings>::default()` plugin to the app.
This will show a window with only the `CameraSettings` resource.

## Fixing cursor grabbing

If you try to click in the gui, you will notice that this causes the cursor to be grabbed.
We need to modify `toggle_cursor` to avoid this, by checking if the cursor is currently over the gui.
A quick search reveals [`EguiWantsInput`](https://docs.rs/bevy_egui/latest/bevy_egui/input/struct.EguiWantsInput.html).
Read the documentation, and ensure that the cursor is only grabbed when the mouse is clicked outside of the gui.
