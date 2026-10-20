# Multiple voxel types

Now we'll add another voxel type.
Rather than creating another voxel type and simply giving it another colour, at this point we are better off using textures.
Otherwise, we'll do a lot of work to hack in the different colours, only to immediately change it.

You might already know that sprites are typically bundles into a single file, rather than a bunch of files for each possible voxel face.
There are good reasons for this:
(sources)

Besides performance reasons, this approach is actually very convenient.
We already saw that meshes also contain so-called UV's.
I skimmed over those before, but here they become relevant.
(Explain UV)

Be aware that often the (0,0) coordinate in an image could either be the top left, or the bottom left.
With increasing y pointing down or up respectively.
Do your own research and determine which way Bevy does it.
That tells you how to define the coordinates in your `TextureAtlasPosition`.

We used to have `Face::uvs`, but if you think about it, that makes no sense.
UV coordinates are determined by the voxel type, not the face.
Initially it was convenient to do it like that because we only had a single voxel type, but now we should move this logic into `TextureAtlas`.

## Using the texture

Find a spritesheet you like somewhere online, and put it in the `assets` folder.

For pixel-art voxel textures you'll probably also want nearest-neighbour texture filtering.
Otherwise, your grass/dirt pixels can become blurred.

## Different faces

Now let's make the grass block have different sides from the top.
It turns out, if we do this naively the texture is upside down.

In Bevy, the vertical axis for the pixels of textures / images, and when sampling textures in a shader, points downwards, from top to bottom. The origin is at the top left.
This is inconsistent with the World-coordinate system used everywhere else in Bevy, where the Y axis points up.
It is, however, consistent with how most image file formats store pixel data, and with how most graphics APIs work.

The easiest way to solve this is by flipping the top and bottom coordinates in `TextureAtlas::uvs`.

## Bevy's built-in TextureAtlas

TextureAtlas: to show how it works I created a simple TextureAtlas here, but we can actually just use Bevy's built-in `TextureAtlas` type.
(You should know this if you read the https://bevy.org/news/introducing-bevy/ :D )
Either continue using the custom one, or switch to Bevy's built-in one.

- TextureAtlasLayout describes how the spritesheet is divided into tiles.
- TextureAtlas selects a particular tile when rendering a Sprite.
- UV coordinates map each tile onto the vertices of your voxel face mesh.

(Potentially incorrect, test later:
```rust
#[derive(Resource)]
struct VoxelTextures {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

fn setup_voxel_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = asset_server.load("textures/voxel_faces.png");

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32), // Tile size in pixels
        4,                // Columns
        4,                // Rows
        None,             // Padding
        None,             // Offset
    ));

    commands.insert_resource(VoxelTextures { image, layout });
}
```
)
