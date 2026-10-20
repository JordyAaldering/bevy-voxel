# Multiple chunks

So far, we have a single chunk at the world origin.
A real voxel world needs to keep track of many chunks, place them in the right locations, and account for voxels that belong to neighbouring chunks.

This introduces a new consideration: **a chunk can no longer generate its mesh using only its own data.** Whether a face is visible may depend on a voxel stored in another chunk.

## Storing the world

Initially, I assumed a `HashMap` would be the best approach for storing chunks, but I still took a moment to look at the alternatives.
Bevy provides its own [hash map implementation](https://docs.rs/bevy/latest/bevy/platform/collections/hash_map/index.html), intended for use in game engines, so I'll use that rather than importing one from the standard library.

Our `VoxelWorld` resource stores chunks in a map, keyed by their `ChunkPosition`.
Each position contains an X and Z coordinate.
We don't need a Y coordinate because chunks currently extend through the entire height of the world and are only placed horizontally.

This is an important distinction: a chunk's position in the map identifies it, while its origin determines where it appears in world space.
For a chunk at coordinate `(x, z)`, its world-space origin is:

```rust
vec3(
    x as f32 * CHUNK_WIDTH as f32,
    0.0,
    z as f32 * CHUNK_WIDTH as f32,
)
```

Multiplying by the chunk width ensures that adjacent chunks line up without overlapping.
The mesh itself is still generated using coordinates local to its chunk; the entity's `Transform` places it in the world.

The world can now look up a neighbouring chunk by adding an offset to a chunk's coordinates.
For example, the chunk to the right is at `(x + 1, z)`, while the chunk behind it is at `(x, z - 1)`.

## Generating multiple chunks

Instead of spawning a single chunk directly, we separate chunk generation from spawning.
The `generate_chunk` function creates and returns the voxel data, while `spawn_world` manages the world and creates the entities that represent its chunks.

For now, the world contains a small grid of nine chunks, with coordinates ranging from `-1` to `1` on both axes.
Their terrain heights vary so that we can see how the chunks fit together.

There is a reason we generate all the chunks before building any of their meshes.
When generating a mesh, we need to know whether a voxel just beyond the edge of the current chunk is solid.
If we start meshing before the neighbouring chunks have been inserted into the world, we won't have that information.

Separating data generation from mesh generation makes this dependency explicit.
Later, when chunks are generated dynamically, we'll need a more deliberate strategy for handling chunks that are not ready yet.

## Smarter meshes

In the previous chapter, we avoided generating faces between solid voxels within the same chunk.
At the boundary, however, we always generated the face because we had no way to inspect the voxel on the other side.

Now we can extend that logic.

When checking a neighbouring voxel, there are two cases:

- **The neighbour is inside the current chunk.** We can inspect it directly.
- **The neighbour is outside the current chunk.** We check whether a neighbouring chunk exists and, if it does, inspect the voxel on its corresponding edge.

For example, the voxel immediately to the right of the current chunk's last X coordinate is at X coordinate zero in the chunk to the right.
The Y and Z coordinates remain unchanged.
The other horizontal boundaries work in the same way.

If the neighbouring voxel is solid, we skip the face.
If it is air, we generate it.

Chunks currently only have horizontal neighbours.
Their top and bottom faces are always generated because we treat those boundaries as the limits of the world.

### What if the neighbouring chunk doesn't exist?

For now, we generate the face whenever the neighbouring chunk is missing.
This treats unknown space as empty, which is simple and lets us render chunks independently.

However, it isn't necessarily what we want in a finished world.
Imagine a chunk being meshed before its neighbour is generated.
Its boundary faces will be included.
Once the neighbour appears, those faces may become hidden, but the existing mesh won't automatically know that it needs rebuilding.

We will encounter a similar problem later when generating structures such as trees near chunk boundaries.
A tree might extend into a neighbouring chunk that hasn't been generated yet.

Ultimately, we'll need to split chunk generation into phases.
A chunk should only advance to a phase when the neighbouring chunks have reached the stage that provides the information it needs.
For mesh generation, that means knowing the neighbouring voxel data.
For structures, it may mean knowing enough of the surrounding terrain to place them consistently.

For now, we'll keep the simple behaviour and return to this problem when we start generating chunks dynamically.

## Debugging chunks with egui

As the world becomes more complex, being able to inspect its state will be increasingly useful.
We already have `bevy-inspector-egui`, but we don't necessarily want to expose every chunk for arbitrary editing.
Changing voxel data would eventually require us to update its mesh as well, and we should be deliberate about how that happens.

Instead, let's create a small custom egui window that lists the loaded chunks.
For each chunk, it shows its coordinates, world-space position, and number of solid voxels.

This is a useful introduction to building custom debugging tools.
The window doesn't modify the world; it simply presents information about the data we're generating.
As the project grows, we can extend it with more useful diagnostics without coupling the editor directly to chunk generation.

With multiple chunks in place, we now have the basic structure of a voxel world.
The next challenge is deciding how to generate and manage that world without assuming that every chunk is available at once.

## Refactoring

So far, we've been using a tuple of usize to represent a voxel's local position.
We can improve this by creating a dedicated struct for the position.
This improves code clarity, as it makes it clear that we are dealing with a voxel's local chunk index.
Also, it enables us to implement `Index<VoxelPosition>` and `IndexMut<VoxelPosition>` for `Chunk`, allowing us to access voxels using `chunk[voxel_pos]` instead of `chunk.voxels[voxel_pos.index()]`.

Implement this yourself, and adjust functions where necessary.
