# Creating a chunk

So far, what we've done was mainly to get an understanding of how to work with Bevy and how to generate meshes.
From this point on, we need to be more careful about our design decisions.

Initially, we might think it is a good idea to have a chunk contain an array of `struct Voxel`, each holding relevant data to that voxel, such as:
 * Position
 * Orientation
 * Links to neighbouring voxels
 * Any attached inventory
 * etc.
However, if we think about this a little bit we realize that this will result in a huge amount of duplication or unnecessary data.
If we consider that our game might contain thousands of voxels, that is suddenly a lot of extra data.
Our voxel array should contain as little data as possible.

Let's search around a bit to see how other people resolve this issue...

https://www.reddit.com/r/VoxelGameDev/comments/1lynvrl/how_do_i_efficiently_store_blocks_in_chunks/
https://gamedev.stackexchange.com/questions/172119/storing-voxels-for-a-voxel-engine-in-c
https://discussions.unity.com/t/designing-a-voxel-structure-around-ecs/789222/4
