# Creating a voxel mesh

In a real voxel game, we don't want to create a separate `Cuboid` for every voxel.
Instead, we want to combine the visible faces of many voxels into a single mesh for each chunk.
This reduces the amount of geometry we need to render.

Before we get to chunks, let's start with a single voxel and build its mesh ourselves.
Along the way, we'll look at how meshes work in general.
Once you understand the underlying concepts, you can use them to generate all kinds of geometry, not just voxels.

## What is a mesh?

A mesh describes the shape of a 3D object.
It consists of vertices, which define points in space, and indices, which define how those points are connected.

A vertex is more than just a position, though.
It can also contain information such as a normal, texture coordinates, or a colour.
These attributes determine how the surface looks when rendered.

Most real-time 3D graphics are built from triangles.
Triangles are simple to render, always lie on a plane, and can be used to approximate almost any surface.
Even a model that looks smooth or curved is typically made up of many small triangles.

A cube has six faces.
Each face is a quadrilateral, which we can split into two triangles:

```text
v2 +---------+ v3
   |        /|
   |      /  |
   |    /    |
   |  /      |
v0 +---------+ v1
```

The diagonal divides the face into two triangles: `(v0, v1, v2)` and `(v2, v1, v3)`.

A cube has eight unique corners, but our mesh will initially contain 24 vertices: four for each of its six faces.
This might seem wasteful, but there's a reason for it that we'll get to later.

For now, the important idea is that **a mesh is data describing geometry and its surface properties**.
It doesn't inherently know what a voxel is, how big a cube should be, or which faces are visible.
Those are decisions we make when generating the mesh.

## Positions and coordinate systems

Before generating geometry, we need to decide where its vertices should be.

Our voxel grid uses integer coordinates to identify each voxel.
A voxel at `(2, 1, 3)`, for example, occupies the space between `(2, 1, 3)` and `(3, 2, 4)`.

```text
             y
             |
        +---------+
       /         /|
      +---------+ |
      |         | +
      |         |/
      +---------+------ x
     /
    z

    Origin: (2, 1, 3)
    Size:   (1, 1, 1)
```

This illustrates an important distinction: **the position of an object is not the same as the positions of its vertices**.

We can define a cube around the origin, with corners ranging from `(0, 0, 0)` to `(1, 1, 1)`, and then translate it to wherever the voxel belongs in the world.
Alternatively, we could generate its vertices directly at their world positions.

Both approaches can work.
Keeping geometry relative to an origin is often convenient because the same geometry can be reused at different positions.
In Bevy, we can also use a `Transform` to position an entity independently of its mesh.

For our voxel, we'll use its grid coordinates as the origin and generate the corners relative to that position.
Each voxel has a size of one unit along each axis.

This approach generalizes beyond cubes.
A sphere, for example, can be generated around its centre, and a terrain mesh can use coordinates relative to the terrain's origin.
The underlying principle is the same: decide which coordinate space your geometry uses, and be consistent.

If you're unfamiliar with Bevy's coordinate system, the [Bevy Cheat Book's coordinate systems section](https://bevy-cheatbook.github.io/fundamentals/coords.html) is a useful reference.

## Constructing triangles

We know where our vertices are, but the renderer still needs to know which vertices form each triangle.
This is what indices are for.

Imagine that we store the four corners of a face in an array.
The indices `0, 1, 2` identify the first triangle, while `2, 1, 3` identify the second.

```text
v2 +---------+ v3
   |       / |
   |     /   |
   |   /     |
   | /       |
v0 +---------+ v1

Triangle 1: 0, 1, 2
Triangle 2: 2, 1, 3
```

Indices refer to entries in the vertex array, not to positions in 3D space.
This distinction lets us reuse vertex data without repeating entire positions for every triangle.

For our cube, each face contributes two triangles.
With six faces, we get twelve triangles in total.

When constructing a mesh, we also need to consider the order in which the vertices are listed.
This is called the **winding order**.

### Winding order and face orientation

A triangle has two sides.
Graphics pipelines commonly use the winding order of its vertices to determine which side is the front.
This allows back-face culling: the renderer can skip triangles facing away from the camera.

For example, looking at a triangle from one side, its vertices might appear in clockwise order.
Looking at it from the other side, they appear counterclockwise.
The order of the indices therefore determines which side is considered the front.

```text
Counterclockwise:       Clockwise:

v2                      v1
| \                      \ \
|  \                      \ \
v0--v1                     v0--v2
```

The exact convention depends on the graphics pipeline and its configuration, so don't rely on intuition alone.
The practical rule is to keep your winding order consistent and ensure that the outside of your mesh is oriented correctly.

This matters especially for voxels because every face has a clear outward-facing direction.
If we generate a face with the wrong winding order, it might disappear when viewed from outside the cube.

It also matters when generating more complex shapes.
If some triangles face inward while others face outward, the mesh may render inconsistently.

## Surface normals

Winding order tells the renderer which side of a triangle is the front, but it doesn't fully describe how the surface should respond to light.
For that, we need **normals**.

A normal is a vector perpendicular to a surface.
For a flat face of a cube, the normal points directly outward:

```text
             +Y
              ↑
         +---------+
        /         /|
       +---------+ |
       |         | +
       |         |/
       +---------+
```

The top face has a normal pointing along positive Y.
The bottom face points along negative Y.
The remaining faces point along positive or negative X and Z.

Normals are used by lighting calculations to determine how light interacts with a surface.
A surface facing directly toward a light generally receives more direct illumination than one facing away from it.

For a flat-shaded cube, all four vertices of a face have the same normal.
Adjacent faces have different normals, which gives the cube its sharp edges.

You might wonder why normals are stored per vertex if they describe a surface.
It's because rendering commonly interpolates vertex attributes across triangles.
Giving neighbouring faces separate vertices lets each face have its own normal, even when their vertices occupy the same spatial position.

For a smooth surface, we can instead use normals that vary across the mesh.
A sphere, for example, often uses normals pointing approximately outward from its centre.
This makes lighting appear smooth even though the geometry consists of flat triangles.

The geometry determines the actual shape.
Normals influence how that shape is shaded.

## Texture coordinates

A mesh can have correct geometry and lighting and still need information about how an image should cover its surface.
This is where **UV coordinates** come in.

UV coordinates associate each vertex with a position in a 2D texture.
They are usually represented by two values, conventionally called U and V.

```text
(0, 0) +---------+ (1, 0)
       |         |
       | texture |
       |         |
(0, 1) +---------+ (1, 1)
```

These coordinates are independent of the mesh's 3D positions.
A vertex at `(0, 1, 0)` could use the UV coordinate `(0.5, 0.5)`, for example.
The first pair describes where the vertex is in 3D space; the second describes which point of the texture maps to it.

For a face using an entire texture, we can assign its four corners to the four corners of the UV square.
The renderer interpolates those coordinates across the triangles and samples the texture accordingly.

The order matters here, too.
UV coordinates must correspond to the same vertices as the positions.
If you assign them in a different order, the texture may appear flipped, rotated, or stretched.

UVs aren't limited to cubes or even flat surfaces.
They are how we map 2D images onto arbitrary 3D geometry.
For example, a character model can use a carefully arranged UV layout to map different parts of its surface onto a single image.

We'll return to texture selection and atlases later.
For now, we only need to understand that UV coordinates describe how a texture is mapped onto the mesh, not the shape of the mesh itself.

## Building our cube

We now have the concepts needed to generate a cube mesh:

- **Positions** describe where the vertices are.
- **Indices** describe which vertices form each triangle.
- **Normals** describe which way the surface faces for lighting.
- **UV coordinates** describe how a texture maps onto the surface.

Each face needs four vertices, six indices, four normals, and four UV coordinates.
We can generate those values for each of the six directions and append them to our mesh data.

```text
                 Voxel
                   |
          +--------+--------+
          |        |        |
        Right     Left     Top
          |        |        |
          +--------+--------+
                   |
             Bottom, Front,
                 Back
                   |
                   v
             Mesh attributes
                   |
                   v
                Bevy Mesh
```

The mesh data is stored in separate arrays because each type of attribute serves a different purpose.
The positions and normals are arrays of vectors, UV coordinates are two-dimensional vectors, and indices are integers referring to vertices.

We then pass this data to Bevy's `Mesh` type.
We specify that the mesh uses a triangle list, meaning every three indices define a triangle, and insert the attributes and indices into the mesh.

You don't need to memorize every Bevy API involved.
What matters is understanding the data we're constructing and how the pieces relate to each other.
Once that is clear, the API becomes a way to express the concepts rather than something to copy without understanding.

One detail worth keeping in mind is that **the number of vertices isn't necessarily the number of unique points in space**.
Our cube has eight unique corners, but we generate 24 vertices because each face needs independent surface attributes.
This is a common pattern in mesh generation, not a special rule for voxels.

## Rendering the mesh

We can now replace Bevy's built-in `Cuboid` with our generated mesh.
The entity still has a transform and a material; only the source of its geometry changes.

When you run the game, the result should look much like it did before.
The difference is that we now control how the geometry is constructed.

Try changing the coordinates of the vertices, adjusting the normals, or changing the UV coordinates.
Observe how each change affects the result.
Reversing the winding order is also a useful experiment: it demonstrates why triangle orientation matters.

These are small changes, but they build an intuition for debugging mesh-generation problems.
When a mesh looks wrong, ask yourself which part of its data is responsible:

- Is the geometry in the right place?
- Are the indices connecting the intended vertices?
- Are the triangles facing the right way?
- Are the normals correct?
- Are the UV coordinates mapped to the intended parts of the texture?

That way of reasoning applies to almost any mesh you generate.

## What's next?

We've generated a mesh for a single voxel, but we're still creating all six faces, regardless of whether they're visible.

Place two voxels next to each other and the faces between them are hidden.
We don't need to render those faces, yet our current approach generates them anyway.

The next step is to generate a mesh from multiple voxels, skipping faces that are covered by neighbouring voxels.
This will give us the foundation for chunk meshes, where we combine many voxels into one mesh and avoid generating geometry that the player can never see.
