# Phase'd chunk generation?

Before we can start adding structures, we need to make sure that the chunk generation is robust enough to handle them.
Namely, whenever we generate a structure at a chunk boundary, we need to make sure that the structure is not cut off by the not-yet-existent neighbouring chunk.

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

---

Implementation steps:

1. **Track chunk generation progress.** Add an explicit phase to `ChunkData` that distinguishes voxel data being available from a mesh being generated. Keep mesh dirtiness separate from generation progress: editing a voxel should request a remesh, not send the chunk back to the voxel-generation phase. When a chunk entity is despawned, retain its voxel data but mark its mesh as needing to be built again when the chunk is loaded.
2. **Separate chunk creation from terrain generation.** Change `VoxelWorld::spawn_chunk` so it creates or reattaches the chunk data and entity without generating terrain inline. Add a generation system that fills chunks which have not reached the voxel-data phase, then advances their phase. Reusing cached chunk data after loading must not overwrite player edits.
3. **Wait for neighbor data before meshing.** Add a mesh-generation system for chunks whose voxel data is ready. It must wait until each of the four horizontal neighboring coordinates has chunk data at least in the voxel-data phase; a missing neighbor is not ready. This check is about voxel data, not whether the neighbor currently has an entity or mesh. Chunks should remain eligible for this system until their mesh is actually created.
4. **Preserve correct border faces and remeshing.** Once a chunk reaches the voxel-data phase, ensure any already-meshed horizontal neighbors are marked for remeshing, since their boundary faces may now be hidden by this chunk. Keep the existing voxel-edit behavior that marks the edited chunk and affected border neighbors dirty. Only remesh chunks whose neighbor data is ready, and do not clear a dirty marker while a chunk is waiting for that condition.
5. **Use a 5x5 data-generation area.** Increase the load radius from one to two chunks and keep the load and retirement ranges consistent. This gives the player a one-chunk buffer so the inner 3x3 area can have all four neighbors ready before meshing. The outer ring is a preparation buffer; it may have voxel data but no mesh until its neighbors are generated.
6. **Integrate the systems in a defined order.** Update the `Update`/`PostUpdate` scheduling so newly requested chunks are created, voxel data is generated, eligible meshes are built, and retired entities are despawned without relying on unspecified deferred-command timing. Avoid building a mesh twice when initial generation and a dirty-remesh request coincide.
7. **Handle unloading and reloading.** Keep voxel data and its voxel-data phase in `VoxelWorld` when a distant chunk entity is retired. On reload, restore the entity and its material, then rebuild its mesh from cached voxel data once neighboring data is ready. Verify that `NeedsDespawn` cancellation still works when the player returns before despawning completes.
8. **Validate the full lifecycle.** Check initial startup and movement across chunk boundaries; confirm the central 3x3 area meshes while the outer ring waits; confirm no missing or duplicate faces at chunk borders as neighbors become ready; and verify that breaking/placing border voxels, unloading, and reloading preserve both correct meshes and edited voxel data.

## Checking the results

Run the game and you'll see: nothing.
At least, not from the player's view.
Using the editor, you should see that meshes at the edges should show some faces, depending on whether the not yet generated chunk beside them has a lower terrain height in some positions.

You might also want to add a nice visualization to the telemetry server to show how many chunks are in each phase.
