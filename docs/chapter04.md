# Creating a chunk

The whole point of creating our own mesh was to more efficiently generate chunks, so let's do that, shall we?
Chunks are at the core of our game, so it's worth first thinking carefully about how we represent our data.

Our first instinct might be to represent each voxel as a struct containing everything associated with it:

- Its position and orientation.
- References to neighbouring voxels.
- Its inventory or other attached data.
- Its type and other properties.

This can work, but it also means storing a lot of information that we don't necessarily need.
Imagine a world containing millions of voxels.
If every voxel stores its own position, we duplicate information that we could derive from its location in the chunk.
If every voxel stores references to its neighbours, we add more data and make changes to the world more complicated.

Every byte of extra data blows up when we have millions of voxels, so we need to be extremely careful about what we store, and where.

I wouldn't be surprised more complicated voxels, like chests or furnaces, will ultimately become entities.
We are working with an Entity Component System, after all.
I'll leave that to a future chapter.

## What's in a chunk?

All a chunk needs to know is what voxels it has, and where.
The _what_ is easy: we can create a `VoxelType` enum, that contains only the type of voxel and nothing else.

```rust
enum VoxelType {
    Air,
    Grass,
}
```

The _where_ is not so straightforward.
We can't waste space storing the voxel's position, but luckily we don't need to: based on the index of a voxel in the chunk's voxel array, we can derive its 3D coordinate within the chunk.

We have two options for storing the voxels:

- A 3D array: `[[[VoxelType; CHUNK_WIDTH]; CHUNK_HEIGHT]; CHUNK_WIDTH]`
- A flat array: `[VoxelType; CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_WIDTH]`

Both have their advantages and disadvantages.
From what I could find, it seems that they _should_ have the same memory layout.
A 3D array is easier to index, but the flat array is simpler to iterate over. (_Need to look more into this._)
Pick whichever you prefer.

To index into the flattnened array from an x, y, and z coordinate, we use this formula:

> `index = x + CHUNK_WIDTH * (y + CHUNK_HEIGHT * z)`

- Moving along x simply increments the index.
- Moving along y means we 'move by a row', which means moving by `CHUNK_WIDTH` entries.
- Moving along z means we 'move by a layer', which means moving by `CHUNK_WIDTH * CHUNK_HEIGHT` entries.

Keep this in mind if you ever iterate over large areas of the chunk.
It is best to put the z loop on the outside, the y loop in the middle, and the x loop on the inside.
Incrementing x by one (y and z remaining the same) is the next entry in memory, which is likely already loaded into the processor's caches.
Putting the loops in the opposite order leads to a lot of jumping around in memory: incrementing z by one (x and y remaining the same), we jump by `CHUNK_WIDTH * CHUNK_HEIGHT` entries every time.

> This results in so-called [_cache misses_](https://www.geeksforgeeks.org/computer-organization-architecture/types-of-cache-misses), and inhibits the use of [_vectorization_](https://en.wikipedia.org/wiki/Single_instruction,_multiple_data).

## Spawning a chunk entity

...

## Generating the chunk mesh

...

## Refactoring meshes

In future, we'll be creating a mesh for each chunk.
For this, it can be convenient to group vertices, indices, normals, and uvs into a single `MeshData` struct.
That keeps the relevant data in one place, and we can add some convenience functions, like `add_quad`.
If you want, extract the mesh data into a separate struct now.

Now might also be a good time to get rid of the `Face` struct.
It was useful for generating a single cube, but now that we're generating many faces, it adds unnecessary complexity.
Its logic can be moved into the `MeshData` instead.
If you put the `MeshData` in a separate file, you can more easily determine which parts are specific to rendering, and should be there.

---

Potentially interesting sources:

- [How do I efficiently store blocks in chunks?](https://www.reddit.com/r/VoxelGameDev/comments/1lynvrl/how_do_i_efficiently_store_blocks_in_chunks/)
- [Storing voxels for a voxel engine in C](https://gamedev.stackexchange.com/questions/172119/storing-voxels-for-a-voxel-engine-in-c)
- [Designing a voxel structure around ECS](https://discussions.unity.com/t/designing-a-voxel-structure-around-ecs/789222/4)

---

## Generating the chunk mesh

In the previous chapter, we generated all six faces of a cube.
Now we need to do that for every voxel in the chunk.

The basic algorithm is straightforward:

1.
Iterate over every voxel in the chunk.
2.
Skip voxels that are air.
3.
For each solid voxel, examine its six faces.
4.
Generate only the faces that need to be visible.

```text
For each voxel:
    If it is air:
        Skip it

    For each of its six directions:
        Check the neighbouring voxel

        If the neighbour blocks this face:
            Skip the face

        Otherwise:
            Add the face to the mesh
```

We use three nested loops to visit every coordinate in the chunk.
For each voxel, we retrieve its type from the flat array.
If it's air, there's nothing to render, so we move on to the next voxel.

For a solid voxel, we examine each of its six directions.
This is where our existing `Face` implementation becomes useful: we can reuse the vertex, index, normal, and UV generation from the previous chapter.

The only difference is that the face vertices now need to be positioned at the voxel's coordinates rather than always at the origin.
A face belonging to the voxel at `(2, 1, 0)`, for example, is offset by that position.

We append the resulting data to the same arrays, building one mesh for the entire chunk instead of one mesh per voxel.

## Removing hidden faces

If we simply generate all six faces for every solid voxel, the result will work, but it will contain unnecessary geometry.

Consider two grass voxels next to each other:

```text
+-------+-------+
|       |       |
| Grass | Grass |
|       |       |
+-------+-------+
        ^
   Shared face
```

The shared face is inside the solid block of voxels.
The player cannot see it from outside, so there's no reason to generate it.

The same applies to every face adjacent to another solid voxel.
We only need the faces that border empty space or, in a more advanced implementation, a transparent voxel that allows us to see through it.

This is often called _face culling_ in voxel meshing.
It is different from the GPU's back-face culling discussed in the previous chapter: here, we're deciding not to create certain faces in the first place.

### Finding a neighbour

To determine whether a face is visible, we need to inspect the voxel immediately next to it.

For example, the neighbour to the right has coordinates `(x + 1, y, z)`, while the neighbour above has coordinates `(x, y + 1, z)`.
The other directions follow the same principle.

Our `Chunk::get_neighbor_voxel` method calculates these coordinates and returns the neighbouring voxel if it lies within the chunk.

Its return type is `Option<VoxelType>` because a neighbour might not exist within the current chunk.
For example, a voxel at `x = 0` has no left neighbour _inside this chunk_.

```text
         Chunk boundary
               |
               v
        +------+-------+
        |      |       |
        | Grass| Grass |
        |      |       |
        +------+-------+
               ^
         Check neighbour
```

There are two cases to consider:

- **The neighbour is inside the chunk.** If it is solid and opaque, the shared face is hidden and we skip it.
If it is air, we generate the face.
- **The neighbour is outside the chunk.** We generate the face for now, because we haven't implemented communication between neighbouring chunks yet.

The second case is important.
Reaching the boundary of one chunk doesn't necessarily mean reaching the edge of the world.
Another chunk may exist on the other side.
We'll deal with that once we start connecting chunks together.

For now, our neighbour check treats any voxel that isn't air as solid and opaque.
If we later add water, glass, or other transparent voxel types, we'll need a more nuanced rule for deciding whether a face should be rendered.

## The result

Run the game and inspect the chunk.
Instead of rendering every face of every grass voxel, we're generating only the exterior faces of the solid region.

This reduces the amount of geometry substantially.
A solid block of N voxels could have 6N faces if we generated every face independently.
By removing shared faces, we avoid generating geometry that would be hidden inside the block.

The exact savings depend on the shape of the voxel region.
A compact solid chunk has many shared faces, while isolated voxels have few or none.
This is one reason why generating chunk meshes is so useful for voxel games.

There is still plenty of room to improve this approach.
We currently create four vertices for every visible face, even when neighbouring faces could share vertex data.
We also haven't considered how to handle transparent materials or neighbours across chunk boundaries.

Those are separate problems, and we don't need to solve them all at once.
The important step is that we now have a chunk data structure and a mesh-generation algorithm that can skip hidden faces.

Next, we can build on this foundation to make chunks interact with each other and form a larger voxel world.
