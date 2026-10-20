# Phase'd chunk generation?

In the previous chapter be briefly discussed the idea of generating chunks in phases.
Now, we will take a first step in doing so.

Currently, when a chunk is created, it is immediately filled with voxels and its mesh is generated.
When a neighbouring chunk is not yet created, we naively render all faces of the voxels at the chunk boundary.
Then, when that chunk is generated, we must generate the mesh again.
(Note that this also occurs if we instead show no faces at the missing chunk's side, because if we don't re-render, then we map have gaps.)

We solve this by splitting chunk generation in two phases:
1. Fill the chunk data with voxels.
2. Only when all neighbouring chunks are at least at phase 1, generate the mesh.

With our current 3x3 chunk generation area, only the chunk the player is standing in could ever be fully generated.
To resolve this for now, we simply increase the generation area to 5x5 chunks.
It might be better to use a separate radius for each phase, but I feel that might become cumbersome when there are many phases.
Follow whichever approach you think is best.
