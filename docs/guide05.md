# Multiple chunks

Initially I assumed a HashMap would be the best approach for storing multiple chunks, but nonetheless I did a quick look around to see if that is true.
Turns out Bevy has its own [implementation](https://docs.rs/bevy/latest/bevy/platform/collections/hash_map/index.html), which is better suited to game engines, so we'll use that.

## Smarterer meshes

Currently, we always draw faces at the edge of a chunk.
If we are trying to get a voxel at the edge, we need to see if there exists a chunk beside it.
If so, we need to check if the corresponding voxel in that chunk is transparent.

We only place chunks horizontally, so whenever the face is up or down we are simply at the build limit, and always have to show that face.

We can further optimize this.
If a chunk at the requested position does not exist, we can assume that it has not yet been necessary to generate it.
Thus, the player won't be able to look from that side anyway, so if a chunk does not exist we also don't draw the faces.

## Egui debugging

Chunks are quite a bit more complex, and realistically we don't want to be able to edit them from inside egui, as we then need to be very careful about what we change and how we reconstruct the mesh.
Instead, this is a perfect opportunity to learn about creating a custom editor window.
For now, we'll just show each chunk's coordinate.
