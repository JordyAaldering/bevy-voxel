# Creating a cube mesh

In a real game, we don't want to create a `Cuboid::new` for every voxel.
Instead, we want to create a single large mesh for each chunk, that shows only the visible faces of each voxel.
To start, we'll generate a mesh for our single voxel.

https://bevy-cheatbook.github.io/fundamentals/coords.html

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
