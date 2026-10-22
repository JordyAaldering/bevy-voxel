# Editor and refactoring

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
