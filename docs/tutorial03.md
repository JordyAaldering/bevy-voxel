# Creating a cube mesh

In a real game, we don't want to create a `Cuboid::new` for every voxel.
Instead, we want to create a single large mesh for each chunk, that shows only the visible faces of each voxel.
To start, we'll generate a mesh for our single voxel.
