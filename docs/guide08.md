# Breaking and placing blocks

To determine where to place or destroy a block we need to determine what we are looking at.
Typically, this is done using 'raycasts'.
But again, because we are in our simple voxel world we can actually do this entirely mathematically.

We use the 'DDA' algorithm.
https://en.wikipedia.org/wiki/Digital_differential_analyzer_(graphics_algorithm)

For debugging purposes we also add an outline to the face we are looking at.
Gizmos are perfect for this.

## Refactoring

Before we continue, some refactoring is in order.
We now need to keep track of the meshes we created.
To do this, we move the created mesh inside the Chunk struct.
Each chunk then becomes responsible for updating its own chunk.
This means we need to move some logic.

## Re-meshing dirty chunks

Meshes are now local to each chunk.
Whenever a change occurs (for example, a block is placed or removed, but this could also be a tree growing), we mark that chunk as 'dirty'.
We don't immediately update the chunk.
Instead, we mark it as dirty, and the `World` becomes responsible for coordinating which chunks update when.
We do this because the world can then decide when it is a good time to re-mesh and to batch multiple meshing operations together.

However, I was not yet sure whether this still applies to ECS systems.
It seems those might naturally expose the desired dirty-tracking.
Time for some googling.

It seems we have a few alternatives.

We can add a marker:
```rust
#[derive(Component)]
struct NeedsRemesh;
```

Or we can make `Chunk` a `Component`, and then use Bevy's `Changed<Chunk>` feature to automatically detect changes.
Because the latter may also respond to changes that do not relate to the mesh, and because it does not respond to changes necessary to neighbouring chunks, I don't think that is the right approach.
However, `NeedsRemesh` seems like a very natural fit for ECS. We anyway already have an entity for the chunk: the mesh itself.

Remember that, if we modify a voxel on the edge of a chunk, we also need to modify the neighbouring chunk.
