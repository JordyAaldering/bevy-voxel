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
