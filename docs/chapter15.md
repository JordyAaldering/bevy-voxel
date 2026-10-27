# Trees, finally!

## Some setup

We only have grass, dirt, and stone, so add new voxel types for logs and leaves.
My spritesheet does not have a leaf texture, so I need to replace it.
Rather than creating another spritesheet, now is a good time to actually split the spritesheet into separate images.

_But I thought that was bad for performance?_

Yes, but we have a secret weapon: [`TextureAtlasBuilder`](https://docs.rs/bevy/latest/bevy/prelude/struct.TextureAtlasBuilder.html).
We already used the `TextureAtlas` before with the entire spritesheet, but we can actually build the atlas dynamically from separate images.
It's a bit more work now, but makes it much easier to add new voxel types later, whilst not giving up any performance.

Another problem is that leaves might be see-through.
We'll ignore that for now, and just pick a texture that is not transparent.
Solving this is actually quite tricky, and needs some shader work, which probably will deserve its own chapter.

## Trees, finally? Finally!

At least, I hope I'll be able to add trees in this chapter.
I'm really looking forward to a less boring world.
