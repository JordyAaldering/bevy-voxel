# Breaking and placing blocks

To determine where to place or destroy a block we need to determine what we are looking at.
Typically, this is done using 'raycasts'.
But again, because we are in our simple voxel world we can actually do this entirely mathematically.

We use the 'DDA' algorithm.
https://en.wikipedia.org/wiki/Digital_differential_analyzer_(graphics_algorithm)
