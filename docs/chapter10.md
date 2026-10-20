# Generating the world

So far we've only manually generated 3x3 chunks.
However, what we want is for the world to be infinitely generated.
Of course, we cannot generate the entire, infinite world all at once.
Instead, we should only generate chunks that are within some distance to the player.

Currently, we naively generate all chunks in a 3x3 square around the player.
It would be better to generate chunks in a circular radius instead, so implement that now if you want.

## Disabling meshes

If the player has been exploring for a while, many chunks/meshes will still be active that the player can't even see.
Whenever a visible mesh is some distance away from the player, we disable that chunk.

Just like the `NeedsRemesh` component, we add a `NeedsDespawn`.

I'll deliberately keep the radius very small (only the chunks directly around the player are generated and kept alive) so that it is clear to see what is going on.

## Tracking state of disabled meshes

Although we don't have any stateful voxels yet, like furnaces or crops, now is a good point to discuss how we'll deal with this in the future.
This is the kind of stuff you need to think about sooner rather than later, because it might influence your design. (In this case it won't)
Turns out, the solution is actually very simple.
We can take this minecraft mod as an example: https://modrinth.com/mod/unloaded-activity.
Store a start time or last-update timestamp, then calculate the current state directly.
For example, a furnace that needs 10 seconds to finish can determine whether its work is complete from elapsed time.
This won't work for more complicated systems, like mob forms or redstone circuits, but I thought this was an interesting share.
It shows how a simple solution can often work very well.
