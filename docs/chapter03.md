# The first mesh

I hope you're warmed up, because this is where the real work starts.
In this chapter we are going to manually generate the mesh for a voxel, rather than using Bevy's built-in `Cuboid` primitive.
In a real voxel game, we don't want to create a separate cuboid for every voxel.
Instead, we want to combine the visible faces of many voxels into a single mesh for each chunk.

When our game has hundreds, or even thousand of voxels, we don't want to spawn a cuboid for every single voxel.
First, this would mean that we generate a lot of faces for voxels that we can't even see.
Second, graphics cards are not optimized for rendering on many small meshes, scattered throughout memory.
They are much better at operating on large, contiguous blocks of data, which is why we want to combine many voxels into a single mesh.
This groups the different data of each voxel by type and puts them next to each other in memory, which is much more efficient for the GPU to process.

## Visualizing meshes

We can visualize the so-called _wireframe_ of a mesh by drawing lines between the vertices of each triangle.
In Bevy, we can do this by adding the `WireframePlugin` to our app.
Then, we insert the configuration globally for all meshes:

```rust
WireframeConfig {
    global: true,
    ..default()
}
```

Run the game, and see what the mesh looks like.
Use the inspector to adjust the scale of the cube, and observe how the wireframe changes.
Then, replace the cube with a `Sphere`, or some other shape, and see how the wireframe looks for that shape.

## But what even is a mesh?

<img align="right" width="30%" src="https://preview.redd.it/missed-out-on-not-having-the-polygon-lara-croft-as-cosmetic-v0-9hcvefmlv6dd1.jpeg?auto=webp&s=e2fff685ba3a16e48241ff2e47b2d7ce7c1f3b9a">

Essentially, a mesh just defines the shape of a 3D object.
This shape is broken up into polygons (triangles), that together form the surface of the shape.
We use triangles because they are the simplest 2D shape, making them the easiest to work and compute with.

In its simplest form, a mesh consists of a list of _vertices_ and a list of _indices_.
The vertices define the corners of the triangles in the mesh.
Each 3-tuple in the list of indices tells which three vertices form a triangle.

To make a triangle, we need three vertices and three indices.
The triangle is then formed by connecting the three vertices in the order specified by the indices.
Say, we have the following vertices: $v_0 = (0, 0)$, $v_1 = (1, 0)$, and $v_2 = (0, 1)$.

```text
| v2
|
|
| v0   v1
+—————————
```

If the indices are `[0, 2, 1]`, then the triangle is formed by connecting $v_0$ to $v_2$, $v_2$ to $v_1$, and $v_1$ back to $v_0$.

```text
| v2            | v2            | v2
|  |            |  | \          |  | \
|  |            |  |  \         |  |  \
| v0   v1       | v0   v1       | v0———v1
+—————————      +—————————      +—————————
```

A square has four corners, but because we can only make triangles, we need to split it into two triangles, resulting in $2 \times 3 = 6$ indices.
However, these two triangles share two vertices, meaning we only need four vertices.
In the previous example, that would mean adding a fourth vertex $v_3 = (1, 1)$ and extending the indices to `[0, 2, 1, 1, 2, 3]`.

Before we get to chunks, let's start with a single cube and build its mesh ourselves.
Along the way, we'll explore how meshes work in general.
The concepts apply to all kinds of geometry, not just voxels.

## Winding order

The order in which we create a triangle matters.
In the previous example, we created a triangle in a clockwise order.
In a counter-clockwise order, the indices would be `[0, 1, 2]`.

This distinction matters: we typically only render one side of a triangle.
We normally don't see the inside of a mesh anyway, so why would we bother rendering both sides.
However, that means we need to be careful about the order in which we create triangles, or some of them might seem invisible.
(Using the wireframe view can help you find these kinds of errors.)

By default, Bevy uses a [counter-clockwise](https://docs.rs/bevy/latest/bevy/mesh/struct.Mesh.html) winding order.
That means the example from before would be invisible, because the triangle is facing the wrong way.
Instead, we should create the cube from the indices `[0, 1, 2, 2, 1, 3]`.

---

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
