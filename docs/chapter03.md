# Creating a voxel mesh

In a real voxel game, we don't want to create a separate `Cuboid` for every voxel.
Instead, we want to generate a single mesh for each chunk, containing only the visible faces of its voxels.
This reduces the amount of geometry we need to send to the GPU.

Before we get to chunks, let's start with a single voxel and build its mesh ourselves.
This gives us a better understanding of what a mesh actually contains.

If you're unfamiliar with Bevy's coordinate system, I recommend reading the [coordinate systems section of the Bevy Cheat Book](https://bevy-cheatbook.github.io/fundamentals/coords.html).

## What makes up a mesh?

A mesh describes the geometry of an object.
At its simplest, it consists of vertices and indices.
- A **vertex** represents a point in space.
- An **index** refers to a vertex, allowing us to construct triangles without storing the same vertex position repeatedly.

Graphics cards only draw triangles, so a rectangle actually exists of two triangles.
We can represent one rectangle using four vertices and six indices:

```text
v2 +---------+ v3
   |        /|
   |      /  |
   |    /    |
   |  /      |
v0 +---------+ v1

Triangles:
(v0, v1, v2)
(v2, v1, v3)
```

The exact ordering of the vertices matters.
Graphics APIs use the winding order of a triangle to determine which side is its front face.
If we get this wrong, faces may appear invisible because their front is facing away from the camera.

A cube has six faces, each consisting of two triangles.
That is a total of 6 x 4 = 24 vertices and 6 x 2 x 3 = 36 indices.
We could define all 24 vertices and 36 indices manually, but that would be repetitive.
Instead, we'll define a `Face` that knows how to generate its own vertices and indices, then use it to construct all six faces of our voxel.

## Representing a voxel

First, let's define a voxel:

```rust
struct Voxel {
    x: usize,
    y: usize,
    z: usize,
}
```

These coordinates identify the voxel in the voxel grid, they describe which cell in a chunk the voxel occupies.

## Defining a face

Every cube has six faces, one facing each direction along the three axes.
Let's represent those directions with an enum `Direction`.

Each vertex also contains a _normal vector_.
This vector describes which way the surface is facing, and is used for lighting calculations.
For a voxel, the normal is always the same as the direction of the face.

This also explains why we need 24 vertices, rather than eight.
Although the corners of a cube are shared between three faces, each face has its own normal.
Thus, although the positions of the vertices are the same, the normals differ, so we need separate vertices for each face.

The front and back directions lie along the Z axis.
Bevy considers positive Z to be forward, so moving backward decreases Z.

For simplicity, we can implement a new function `Direction::normal` that return the appropriate unit vector for its direction.

Now we can define a face by its origin and direction:

```rust
struct Face {
    origin: Vec3,
    direction: Direction,
}
```

The origin is the voxel's position in local space in the chunk (or in this case, the world).
The direction tells us which of the voxel's six faces we're constructing.

## Generating vertices

Each face has four corners.
Their positions depend on the face's direction and the voxel's origin.

For example, the top face lies at `y + 1`, while the bottom face lies at `y`.
The other four faces follow the same principle, using the appropriate combination of the voxel's minimum and maximum coordinates.

Our `Face::vertices()` method returns the four corner positions in a consistent order:

```rust
fn vertices(&self) -> [Vec3; 4] {
    // Return the four corners of this face,
    // in the correct winding order.
}
```

The actual implementation handles all six directions.
Notice that we generate positions relative to the voxel's origin rather than hard-coding the coordinates of the voxel in the world.

This means the same face-generation logic works regardless of where the voxel is placed.

## Turning vertices into triangles

A face is a quadrilateral, but our mesh is made of triangles.
We need to tell the renderer which vertices form each triangle.

We do this with indices.
Given four vertices, we can construct two triangles using six indices:

```text
v2 +---------+ v3
   |       / |
   |     /   |
   |   /     |
   | /       |
v0 +---------+ v1

Indices:
0, 1, 2
2, 1, 3
```

The indices refer to positions in the vertex array, not coordinates in 3D space.
For example, index `0` refers to the first vertex we added.

Our `Face::indices()` method accepts a starting index:

```rust
fn indices(&self, start: u32) -> [u32; 6]
```

Why do we need `start`? Because our voxel has six faces, and each face appends four vertices to the same array.
The second face must refer to vertices `4` through `7`, the third to `8` through `11`, and so on.

The starting index lets us reuse the same triangle pattern for every face.
It also ensures that each triangle uses the correct winding order, so the outward-facing side of the cube is treated as the front.

## Normals and UV coordinates

Vertices and indices describe the shape of our mesh, but they aren't the only information a renderer needs.
We also want the voxel to respond to lighting and display a texture.

### Normals

A normal describes the direction a surface faces.
Lighting calculations use normals to determine how light interacts with a surface.

For our cube, all four vertices of a face share the same normal.
A top face, for example, has a normal pointing straight up:

```text
       Normal: +Y
          ↑
     +---------+
    /         /|
   +---------+ |
   |         | +
   |         |/
   +---------+
```

We can implement `Face::normals()` by repeating the face's direction normal four times.

This gives the renderer the information it needs to shade the face correctly.
Later, when we generate meshes for more complex shapes, normals will become more interesting.

### UV coordinates

A mesh's geometry is independent of its texture.
To tell the renderer how to apply a texture, each vertex also needs a pair of UV coordinates.

UV coordinates describe positions on a 2D texture.
Both axes typically range from `0.0` to `1.0`, regardless of the texture's pixel dimensions.

```text
(0, 0) +---------+ (1, 0)
       |         |
       | texture |
       |         |
(0, 1) +---------+ (1, 1)
```

Each vertex receives a UV coordinate corresponding to a point on the texture.
The renderer interpolates those coordinates across the triangles, allowing the texture to cover the entire face.

For a face using the whole texture, we can use these four coordinates:

```rust
[
    Vec2::new(0.0, 0.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
]
```

The order must correspond to the order of the face's vertices.
Otherwise, the texture may appear rotated or distorted.

So far, every face uses the entire texture.
That works for a simple test texture, but voxel games usually have several block textures packed into a single image.
This is where a texture atlas comes in.

## Using a texture atlas

A texture atlas is a single image containing multiple smaller textures.
Instead of loading a separate image for every block type, we can store them together and select the part we want when rendering a face.

For example, imagine an atlas containing four tiles:

```text
          U
          0       0.5       1
          +---------+---------+
    0     |         |         |
          |  Grass  |  Stone  |
          |         |         |
   0.5    +---------+---------+
          |         |         |
          |  Dirt   |  Sand   |
          |         |         |
    1     +---------+---------+
          V
```

Each tile occupies a quarter of the atlas.
The UV coordinates for a face using the grass tile must therefore cover only the corresponding region, rather than the entire image.

For example, if the grass tile occupies the top-left quarter, its UV coordinates cover the range `0.0` to `0.5` on both axes.
We map the four face vertices to the four corners of that region.

```text
Atlas UVs:              Face UVs:

(0, 0) +----+ (0.5, 0)  (0, 0) +----+ (1, 0)
       |Grass|                 |    |
       |     |                 |    |
(0, .5)+-----+          (0, 1) +----+ (1, 1)
```

The face still has four vertices and two triangles.
Only the UV coordinates change.

In this chapter, we'll use a small custom `TextureAtlas` implementation to demonstrate how the mapping works.
The important parts are:

- **Atlas layout:** describes how the image is divided into tiles.
- **Tile selection:** identifies which tile a face should use.
- **UV mapping:** converts the selected tile's boundaries into UV coordinates for the face.

Keeping these concepts separate makes the implementation easier to understand.
It also means we can change the atlas layout or select a different tile without changing the mesh geometry.

Bevy already provides built-in types for texture atlases.
In particular, `TextureAtlasLayout` describes the layout, while `TextureAtlas` identifies a selected tile when used with supported rendering components.
Once you understand how the mapping works, you can switch to Bevy's built-in implementation rather than maintaining a custom one.
See the [Bevy documentation](https://docs.rs/bevy/latest/bevy/prelude/struct.TextureAtlasLayout.html) for details.

## Building the mesh

We now have everything needed to construct a voxel mesh: vertices, indices, normals, and UV coordinates.

Our `build_mesh()` function creates arrays to hold this data, then iterates over the six directions.
For each direction, it creates a `Face` and appends that face's data to the corresponding arrays.

Conceptually, the process looks like this:

```text
Voxel
  |
  +-- Right face  -- vertices, indices, normals, UVs
  +-- Left face   -- vertices, indices, normals, UVs
  +-- Top face    -- vertices, indices, normals, UVs
  +-- Bottom face -- vertices, indices, normals, UVs
  +-- Front face  -- vertices, indices, normals, UVs
  +-- Back face   -- vertices, indices, normals, UVs
                         |
                         v
                       Mesh
```

Each face contributes four vertices and six indices.
For a complete cube, that gives us:

| Data | Number of entries |
|---|---:|
| Positions | 24 |
| Indices | 36 |
| Normals | 24 |
| UV coordinates | 24 |

You might wonder why we store 24 vertices when a cube has only eight unique corners.
Each face needs its own vertices because attributes such as normals and UV coordinates can differ at the same spatial position.
Sharing a vertex between faces would prevent us from assigning different per-face values to it.

Once we've collected the arrays, we create a Bevy `Mesh`:

```rust
let mut mesh = Mesh::new(
    PrimitiveTopology::TriangleList,
    RenderAssetUsages::default(),
);
```

`PrimitiveTopology::TriangleList` tells Bevy to interpret each group of three indices as a triangle.
We then insert the position, normal, and UV attributes, followed by the index buffer.

`RenderAssetUsages` controls where the mesh data is retained for rendering and asset access.
You don't need to understand its details yet; the [Bevy documentation](https://docs.rs/bevy/latest/bevy/asset/struct.RenderAssetUsages.html) explains the available options.

The important part is that the mesh is just the result of combining the data we've generated.
Bevy doesn't need to know that it came from a voxel or that we built it face by face.

## Rendering our voxel

Finally, we can replace the built-in `Cuboid` in `spawn_voxel()` with our generated mesh:

```rust
commands.spawn((
    Mesh3d(meshes.add(build_mesh(&voxel))),
    Transform::from_translation(voxel.origin()),
));
```

Run the game.
The voxel should look the same as before, even though we're now generating its geometry ourselves.

Try changing the voxel's coordinates, modifying the normals, or changing the UV coordinates.
These experiments help demonstrate how each part of the mesh affects the result.
You can also reverse the triangle winding order to see what happens when the faces point the wrong way.

## What's next?

We can now generate a mesh for an individual voxel, but we're still generating all six faces.
If we place two voxels next to each other, the faces between them are hidden, yet our meshes still contain them.

Those hidden faces waste memory and rendering work.
The next step is to combine multiple voxels into a chunk mesh and omit faces that are covered by neighbouring voxels.
That's where this basic face-generation code starts to become useful.
