# Creating a chunk

So far, we've mainly focused on learning how Bevy works and how to generate a mesh.
Now we're starting to build the foundation of the voxel engine itself, so it's worth thinking more carefully about how we represent our data.

## Storing voxels

Our first instinct might be to represent each voxel as a struct containing everything associated with it:

- Its position and orientation.
- References to neighbouring voxels.
- Its inventory or other attached data.
- Its type and other properties.

This can work, but it also means storing a lot of information that we don't necessarily need.
Imagine a world containing millions of voxels.
If every voxel stores its own position, we duplicate information that we could derive from its location in the chunk.
If every voxel stores references to its neighbours, we add more data and make changes to the world more complicated.

Instead, let's start with a simpler approach: **store only the information we need to identify each voxel**.

For this chapter, a voxel is either air or grass:

```text
Air     = empty space
Grass   = solid voxel
```

We represent these options using a `VoxelType` enum.
The chunk stores an array of these values rather than a separate struct for every voxel.

This is enough for our current needs.
A voxel's position is implicit in its location in the array, and we can calculate its neighbours when we need them.

This doesn't mean that every voxel engine should store only a type.
Some games need per-voxel state, and others store additional data separately.
The important thing is to consider what information you actually need and avoid storing the same information in multiple places without a good reason.

If you'd like to explore alternative approaches, these discussions are useful starting points:

- [How do I efficiently store blocks in chunks?](https://www.reddit.com/r/VoxelGameDev/comments/1lynvrl/how_do_i_efficiently_store_blocks_in_chunks/)
- [Storing voxels for a voxel engine in C](https://gamedev.stackexchange.com/questions/172119/storing-voxels-for-a-voxel-engine-in-c)
- [Designing a voxel structure around ECS](https://discussions.unity.com/t/designing-a-voxel-structure-around-ecs/789222/4)

## Representing a chunk

A chunk is a fixed-size region of the voxel world.
Rather than creating an entity for every voxel, we store the voxels together and generate a mesh for the chunk.

For now, our chunk is only three voxels wide, four voxels tall, and three voxels deep.
These dimensions are defined by `CHUNK_WIDTH` and `CHUNK_HEIGHT`.

```text
          Chunk: 3 × 4 × 3

             y
             ↑
          +-------+
         /       /|
        +-------+ |
        |       | |
        |       | +
        |       |/
        +-------+----→ x
       /
      z
```

The chunk contains 3 x 4 x 3 = 36 voxels.
We initialize them all as air, then fill the bottom half with grass.

### A flat array

Although a chunk is three-dimensional, we store its voxels in a one-dimensional array.
This might seem odd at first, but it keeps the representation simple and stores the voxel values contiguously.

The challenge is mapping three coordinates to a single array index.

We use this formula:

> index = x + CHUNK_WIDTH * (y + CHUNK_HEIGHT * z)

Here, W is the chunk width and H is its height.
In our implementation, those are `CHUNK_WIDTH` and `CHUNK_HEIGHT`.

The idea is to lay out the coordinates in a consistent order:

- Moving along X advances one array entry.
- Moving along Y advances by one row, which contains `CHUNK_WIDTH` entries.
- Moving along Z advances by one layer, which contains `CHUNK_WIDTH * CHUNK_HEIGHT` entries.

For our chunk, the first few entries are arranged like this:

```text
Coordinates       Array index

(0, 0, 0)    →        0
(1, 0, 0)    →        1
(2, 0, 0)    →        2
(0, 1, 0)    →        3
(1, 1, 0)    →        4
(2, 1, 0)    →        5
...
(0, 0, 1)    →       12
```

X changes fastest, followed by Y and then Z.
This is one common way to lay out a three-dimensional grid in a flat array.
Other layouts are possible, but whichever one you choose, keep the indexing formula consistent.

The `Chunk::index` method implements this mapping.
We use it both when filling the chunk and when retrieving voxels during mesh generation.

This is a useful general technique whenever you need to represent multidimensional data in a flat array.

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

## Refactoring meshes

In future, we'll be creating a mesh for each chunk.
For this, it can be convenient to group vertices, indices, normals, and uvs into a single `MeshData` struct.
That keeps the relevant data in one place, and we can add some convenience functions, like `add_quad`.
If you want, extract the mesh data into a separate struct now.

Now might also be a good time to get rid of the `Face` struct.
It was useful for generating a single cube, but now that we're generating many faces, it adds unnecessary complexity.
Its logic can be moved into the `MeshData` instead.
If you put the `MeshData` in a separate file, you can more easily determine which parts are specific to rendering, and should be there.
