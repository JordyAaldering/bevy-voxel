# Multiple chunks

Initially I assumed a HashMap would be the best approach for storing multiple chunks, but nonetheless I did a quick look around to see if that is true.
Turns out Bevy has its own [implementation](https://docs.rs/bevy/latest/bevy/platform/collections/hash_map/index.html), which is better suited to game engines, so we'll use that.

## Smarterer meshes

Currently, we always draw faces at the edge of a chunk.
If we are trying to get a voxel at the edge, we need to see if there exists a chunk beside it.
If so, we need to check if the corresponding voxel in that chunk is transparent.

We only place chunks horizontally, so whenever the face is up or down we are simply at the build limit, and always have to show that face.

If a chunk is next to a chunk that has not been generated yet we have a more complicated situation.
We'll have a similar problem laten when we start generating structures like trees at the edge of a chunk.
Ultimitely: we'll split chunk generation in multiple phases, each requiring that all neighbouring chunks are all at least at the same phase before this chunk can continue with the next.
In this case, that would mean that the height data of neighbouring chunks has to be known before we can decide which faces we need to draw.
https://gamedev.stackexchange.com/questions/180163/how-do-minecraft-know-where-villages-buildings-are-if-the-village-is-not-genera
For now, however, we'll just draw all faces.

## Egui debugging

Chunks are quite a bit more complex, and realistically we don't want to be able to edit them from inside egui, as we then need to be very careful about what we change and how we reconstruct the mesh.
Instead, this is a perfect opportunity to learn about creating a custom editor window.
For now, we'll just show each chunk's coordinate.
