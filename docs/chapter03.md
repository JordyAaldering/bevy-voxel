# Creating a voxel mesh

TODO: first, add wireframe plugin to show how meshes are constructed for the cuboid.

In a real voxel game, we don't want to create a separate `Cuboid` for every voxel.
Instead, we want to combine the visible faces of many voxels into a single mesh for each chunk.
This reduces the amount of geometry we need to render.

Before we get to chunks, let's start with a single cube and build its mesh ourselves.
Along the way, we'll explore how meshes work in general.
The concepts apply to all kinds of geometry, not just voxels.

## What is a mesh?

A mesh describes the shape of a 3D object.
It consists of vertices, which define points in space, and indices, which define how those points are connected.

A vertex can contain more than just a position.
It can also store information such as a normal, texture coordinates, or a colour.
These attributes determine how the surface looks when rendered.

Most real-time 3D graphics are built from triangles.
Triangles are simple to render and can be used to approximate almost any surface.
Even models that look smooth or curved are typically made up of many small triangles.

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

A cube has eight unique corners, but our mesh will contain 24 vertices: four for each of its six faces.
This might seem wasteful, but there's a reason for it that we'll get to later.

The important idea is that **a mesh is data describing geometry and its surface properties**.
It doesn't inherently know what a voxel is or which faces are visible.
Those are decisions we make when generating the mesh.

## Defining the faces

Our cube occupies the space between `(0, 0, 0)` and `(1, 1, 1)`.
We'll generate its vertices directly in this coordinate range and leave the mesh at the world origin.

Each face lies along one of the three coordinate axes.
We can represent the six possible directions with an enum:

```text
          Top (+Y)
             ↑
             |
Left (-X) ← [ ] → Right (+X)
             |
             ↓
         Bottom (-Y)

    Front (+Z) / Back (-Z)
```

The front and back faces lie along the Z axis.
In Bevy, positive Z points forward, so moving backward decreases the Z coordinate.

Rather than defining six unrelated sets of vertices, we can represent a face by its direction and generate its four corners based on that direction.
For example, the top face lies at `y = 1`, while the bottom face lies at `y = 0`.
The other faces follow the same principle.

This is a useful general approach to procedural geometry: identify what varies, represent it as data, and use shared logic to generate the geometry.
Here, the direction determines which four corners make up the face.

The `Face::vertices()` method in the code handles the six possible directions.
Pay attention to the order in which the vertices are returned, because it affects how the triangles are constructed.

## Connecting vertices with indices

Knowing the positions of the vertices isn't enough.
The renderer also needs to know which vertices form each triangle.
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

Indices refer to entries in the vertex array, not positions in 3D space.
This lets us describe triangles without repeating the full vertex data for every triangle.

Our cube has six faces, each consisting of two triangles, giving us twelve triangles in total.

The implementation generates four vertices for each face and appends them to a shared array.
The `start` argument passed to `Face::indices()` tells the face where its vertices begin in that array.

```text
Face 1: vertices  0–3   indices refer to 0–3
Face 2: vertices  4–7   indices refer to 4–7
Face 3: vertices  8–11  indices refer to 8–11
...
Face 6: vertices 20–23  indices refer to 20–23
```

The same index pattern works for every face.
We simply offset it by the number of vertices already added.

### Winding order

The order in which we list a triangle's vertices matters.
This is called its _winding order_.

Graphics pipelines commonly use winding order to determine which side of a triangle is the front.
This allows back-face culling: the renderer can skip triangles facing away from the camera.

```text
Counterclockwise:   Clockwise:

v2                  v1
| \                 | \
|  \                |  \
v0--v1              v0--v2
```

The practical rule is to keep the winding order consistent and ensure that the outside of your mesh is oriented correctly.
If a face has the wrong winding order, it might disappear when viewed from outside the cube.

In our implementation, the vertices are ordered so that the triangles face outward.
The indices then split each face into two triangles while preserving that orientation.

This is worth understanding when generating any mesh.
If a surface unexpectedly disappears, checking the vertex order and triangle winding is a good place to start.

## Surface normals

Winding order determines the front of a triangle, but it doesn't fully describe how the surface should respond to light.
For that, we need **normals**.

A normal is a vector perpendicular to a surface.
For a flat face of a cube, the normal points directly outward.

Each of the cube's six faces has a normal pointing along one of the coordinate axes.
The front and back faces point along positive and negative Z.
Together, these six directions give us the normals for the cube.

Normals are used by lighting calculations to determine how light interacts with a surface.
A surface facing toward a light generally receives more direct illumination than one facing away from it.

For a flat-shaded cube, all four vertices of a face have the same normal.
Adjacent faces have different normals, which gives the cube its sharp edges.

Why store normals per vertex if they describe a surface? Because a vertex contains a complete set of attributes, and the renderer interpolates those attributes across triangles.
Giving neighbouring faces separate vertices lets each face have its own normal, even when their vertices occupy the same position in space.

For a smooth surface, normals can instead vary across the mesh.
A sphere, for example, often uses normals pointing approximately outward from its centre.
This makes lighting appear smooth even though the geometry consists of flat triangles.

The geometry determines the shape; normals influence how that shape is shaded.

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
The first describes where the vertex is in 3D space; the second describes which point of the texture maps to it.

In our cube, each face uses the entire texture.
Its four vertices map to the four corners of the UV square.
The renderer interpolates those coordinates across the triangles and samples the texture accordingly.

The order matters here, too.
UV coordinates must correspond to the same vertices as the positions.
If you assign them in a different order, the texture may appear flipped, rotated, or stretched.

UV mapping works for arbitrary geometry, not just cubes.
A character model, for example, can use a carefully arranged UV layout to map different parts of its surface onto an image.

We'll explore more advanced texture mapping later.
For now, the important distinction is that UV coordinates describe how a texture maps onto a mesh, not the shape of the mesh itself.

## Building the mesh

We now have the concepts needed to generate our cube:

- **Positions** describe where the vertices are.
- **Indices** describe which vertices form each triangle.
- **Normals** describe which way the surface faces for lighting.
- **UV coordinates** describe how a texture maps onto the surface.

Each face contributes four vertices, six indices, four normals, and four UV coordinates.
We generate these values for each of the six directions and append them to the corresponding arrays.

```text
             Cube
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

For our cube, the arrays contain:

| Attribute | Number of entries |
|---|---:|
| Positions | 24 |
| Indices | 36 |
| Normals | 24 |
| UV coordinates | 24 |

You might wonder why we store 24 vertices when a cube has only eight unique corners.
Each face needs independent surface attributes.
For example, the top and front faces have different normals at their shared corner, so they need separate vertices.

More generally, a vertex is a complete set of attributes, not just a position.
Two vertices can occupy the same point in space while still being different vertices because their normals, UV coordinates, or other attributes differ.

Once we've generated the arrays, we create a Bevy `Mesh`, specify that it uses a triangle list, and insert the attributes and indices.

`PrimitiveTopology::TriangleList` tells Bevy to interpret each group of three indices as a triangle.
`RenderAssetUsages` controls where mesh data is retained for rendering and asset access; you can refer to the [Bevy documentation](https://docs.rs/bevy/latest/bevy/asset/struct.RenderAssetUsages.html) for details.

You don't need to memorise every Bevy API involved.
What matters is understanding the data we're constructing and how the pieces relate to each other.
The API is simply how we express these concepts in Bevy.

## Rendering the mesh

We can now replace Bevy's built-in `Cuboid` with our generated mesh.
The entity still has a transform and a material; only the source of its geometry changes.

Since we've generated the vertices directly in the range from zero to one, we don't need a separate voxel position or a translation to place the mesh.
It already sits at the world origin.

Run the game.
The result should look much like it did before, even though we're now generating the geometry ourselves.

Try changing the vertex positions, adjusting the normals, or changing the UV coordinates.
Observe how each change affects the result.
Reversing the winding order is also a useful experiment: it demonstrates why triangle orientation matters.

When a mesh looks wrong, ask yourself which part of its data is responsible:

- Is the geometry in the right place?
- Are the indices connecting the intended vertices?
- Are the triangles facing the right way?
- Are the normals correct?
- Are the UV coordinates mapped as intended?

This way of reasoning applies to almost any mesh you generate.

## What's next?

We've generated a mesh for a single voxel, but we're still creating all six faces, regardless of whether they're visible.

Place two voxels next to each other and the faces between them are hidden.
We don't need to render those faces, yet our current approach would generate them anyway.

The next step is to generate a mesh from multiple voxels, skipping faces that are covered by neighbouring voxels.
This will give us the foundation for chunk meshes, where we combine many voxels into one mesh and avoid generating geometry that the player can never see.
