# Creating a player character

Adding a first person player, and especially the necessary physics, might seem daunting.
But it turns out that this is not actually as scary as it sounds in our simple voxel world.

First, we'll need some convenience functions.
For the physics, we'll definitely want to be able to look up arbitrary voxels in the world.
So add a `get_voxel(x: i32, y: i32, z: i32) -> Option<VoxelType>` function to `World`.
Use the x and z coordinate to determine the coordinate of the voxel, and then adjust the x and z coordinates to chunk-local coordinates.
We don't place chunks vertically so y can remain unchanged, just check that it remains in bounds.

## Player instance

Create a player struct.
It would be convenient to still be able to go back to freecam mode, so we also add an enum `CameraMode`.

## Fixed Update

For physics, we need to separate player input from the physics calculation.
We **must** run the input function every frame, because we might miss inputs.
But the physics simulation is typically run during the `FixedUpdate` schedule.

TODO: Implement physics as a plugin instead?
