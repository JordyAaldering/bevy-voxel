# Performance debugging, editors, and refactoring

At this point, I would like to see some performance statistics.
What is my framerate? How many chunks are currently loaded? What is my memory usage? etc.
However, I don't want to put all of this in the game view.

![](https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fgamepedia.cursecdn.com%2Fminecraft_gamepedia%2F1%2F18%2FDebugScreen.png&f=1&nofb=1&ipt=c6355da97e9a77c3a9d119ca871bf3f97922691cd487a843e0e9c61cad8e323d&ipo=images)

I find this very hard to read.
Rather, some plots and interactivity would be much nicer.

What we can do is to a browser-based debug dashboard.
- Bevy exposes selected metrics and debug settings to a WebSocket.
- A browser dashboard displays live graphs, counters, etc.

https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs
https://docs.rs/axum/latest/axum/extract/ws/

```bash
cargo add axum --features ws
cargo add tokio --features rt-multi-thread,macros,net,sync,time
cargo add serde --features derive
cargo add serde_json
```

- `axum` is a web framework for Rust, which we can use to serve a dashboard.
- `tokio` is an async runtime for Rust, which we need to run the web server outside the Bevy game loop. Otherwise, the web server would block the game loop.
- `serde` and `serde_json` are used to serialize the metrics data to JSON, which can be easily consumed by the browser dashboard.

You can really take this quite far, and implement a full-featured dashboard with multiple pages, graphs, etc.
I'm no web developer, so I'll keep it simple, but feel free to add more features to the dashboard at this point!

You could even go a step further and make the game configurable from the dashboard, for example for showing gizmos.
However, I prefer to keep the dashboard read-only, and instead use the Bevy `bevy-inspector-egui` plugin for that.

## Feature gate

If you ever share your game, you probably don't want to ship the debug dashboard with it.
We can gate the debug dashboard, and the necessary dependencies, behind a feature flag.

https://doc.rust-lang.org/cargo/reference/features.html

I omit it for now, because I want to avoid recompilations when enabling/disabling certain flags.

## Toggle-able gizmos

I'll use the `bevy-inspector-egui` again to add a toggle for showing gizmos, just like we did for the camera settings.
To avoid a blowup of the number of egui windows, I'll group all debug settings into a single window: `DebugSettingsView`.

This is a bit of a pain, because I still want CameraSettings and DebugGizmos to be separate resources: the player_look function should not have access to DebugGizmos; that's just poor design.
However, I also don't want to have two (or more) separate egui windows for the debug settings, because that would be a mess.
We could manually create a window, but that would also mean manually implementing the sliders etc.
Luckily, we can let the window of each resource be automatically drawn.
Then, we can use a similar approach as we did for the chunks window.

```rust
fn debug_settings_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };

    let mut ctx = egui_context.clone();

    egui::Window::new("Settings").show(ctx.get_mut(), |ui| {
        ui.heading("Camera Settings");
        ui.push_id("camera_settings", |ui| {
            bevy_inspector::ui_for_resource::<CameraSettings>(world, ui);
        });

        ui.separator();

        ui.heading("Debug Gizmos");
        ui.push_id("debug_gizmos", |ui| {
            bevy_inspector::ui_for_resource::<DebugGizmos>(world, ui);
        });
    });
}
```

## A real editor

Whilst looking through the documentation: https://github.com/jakobhellermann/bevy-inspector-egui/blob/main/README.md, I found out about `egui_dock`.
It gives your game a real editor, just like you would find in Unity, for example.
Feel free to skip this step, but I think it is worth a look.

![](https://raw.githubusercontent.com/jakobhellermann/bevy-inspector-egui/main/docs/images/egui_dock.png)

The `bevy-inspector-egui` plugin already has an example for using `egui_dock`: https://github.com/jakobhellermann/bevy-inspector-egui/blob/main/crates/bevy-inspector-egui/examples/integrations/egui_dock.rs.

We can now also add an editor-camera, that we can freely move around in the world without moving the player.
That allows us to inspect the world from different angles and to check if chunks are generated as expected, and is especially useful for debugging.

Bevy 0.20 added a new Pan Orbit Camera: https://bevy.org/news/bevy-0-20/#pan-orbit-camera.
That is perfect for the editor camera.

## Take some time to refactor

Ideally you've been refactoring your code along the way, but if not, now is a good time to do so.
Take 30 minutes to an hour to refactor your code.
To be honest, whilst writing this guide I have not been doing so, and I know it will bite me in the ass later.
I feel like we are now in a nice place; we have a player, infinite terrain generation, and a nice editor to inspect the world.
The next step will be a big one: generating chunks in phases, which I suspect will require changes in multiple places, so we should refactor before that point.

Split logic into files, ensure a good separation of concerns, and make sure your code is readable and maintainable.
I also like to draw a diagram of all of my structures and systems, in which files they live, and how they depend on each other.
Ideally, this should result in a tree with as little circular dependencies as possible.
Use post-it notes, or maybe a whiteboard, if possible; this also lets you easily think about the design and move things around.
It's actually really convenient to visually and interactively/physically think about your code!
Take a close look over all the systems and resources, and see if you can improve the design.
Are resources running in the right order, or are dependencies missing?

Now would also be a good time to add proper error messages.
Similar to the popular anyhow crate, `BevyError` now provides an ergonomic way to attach extra context to an error using the context method, which also allows creating a `Result<T, BevyError>` from an `Option<T>`.
This makes it easier to trace back errors with human-readable messages without looking at verbose backtraces.
https://bevy.org/news/bevy-0-20/#bevy-error-context-messages

At this point, you should understand all the code you have written.
If not, go back through the chapters and remember how everything works, and this time add some documentation to remember in the future.
If you plan on making a real game, now might also be a good time to add some additional voxels and simple features, or to improve the terrain generation.
You'll be looking at this for a long time; and it's always more fun if it actually looks nice.
