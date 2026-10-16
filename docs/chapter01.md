# Project setup

A new version of Bevy containing breaking changes to the API is released approximately once every 3 months, so follow the Bevy [quick start](https://bevy.org/learn/quick-start) to get a simple project up-and-running.

Bevy has a whole bunch of predefined [schedules](https://deepwiki.com/bevyengine/bevy/3.1-app-lifecycle-and-schedules), including but not limited to:
- `PreStartup`
- `Startup`
- `PostStartup`
- `First`
- `Update`
- `FixedUpdate`
- `PostUpdate`

> Exercise: For the following functions, think about which schedule they should run in. spawn_global_illumination, player_physics, render_ui, move_player, spawn_zombie, advance_time_of_day, despawn_unloaded, send_performance_counters_event

Ensure you have the `DefaultPlugins` plugin.
Add the `Startup` and `Update` systems to the app.
`Startup` runs only once at the start, we use it so spawn a cuboid, directional light, and a camera.
In `Update` we'll continuously handle user input.

```rust
use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (
        ))
        .add_systems(Update, (
        ))
        .run()
}
```

# Our first 'voxel'

Spawn a cuboid, directional light, and a camera.

# Moving around

First, we need to confine the mouse cursor to the window, and hide it.

Then, we implement two functions: one for moving the camera with the keyboard, and one for looking around with the mouse.

Schedules are not the only way to control when systems run.
Using `run_if`, we ensure that these functions only run when the window is focused.
