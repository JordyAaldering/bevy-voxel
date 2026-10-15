# Terrain Generation

Rather than using the old-school Perlin noise algorithm, I'll opt for the OpenSimplex2 algorithm.
In particular, I select the OpenSimplex2S variant (https://github.com/KdotJPG/OpenSimplex2/blob/master/java/OpenSimplex2S.java).
Instead of evaluating the BCC lattice precisely, it applies a geometric "skewing" factor to the coordinates before processing them.
This produces slightly smoother results, but is much faster.
Once we combine multiple noise layers, and since our world is blocky anyway, this difference should not be visible.

This is one of those cases where you can definitely skip this step and simply use a Perlin noise or Simplex crate, or whatever you would want to use.

I'll go with OpenSimplex2S.
It seems that it is slightly slower than OpenSimplex2, but produces slightly nicer output.

https://github.com/KdotJPG/OpenSimplex2
https://lib.rs/crates/opensimplex2

Basic idea:
1. Place gradient vectors on a lattice.
2. Find nearby lattice points.
3. Compute gradient contributions.
4. Blend them smoothly.

```
sample position
 v
locate nearby lattice vertices
 v
generate pseudorandom gradients
 v
weight by distance
 v
sum contributions
 v
noise value
```

There's 2d, 3d, and 4d variants of the algorithm.
The 2d variant is the simplest to start with, and we can use it to determine the height of the world.
Later, once we generate caves, we can use the 3d variant.
We likely won't need the 4d variant; its fourth dimension is typically used for time-varying effect, like cloud movement, or animations and procedural textures.

There's very little explanation of the actual algorithm, so we'll have to make do with the source code and descriptions of the Simplex and Perlin algorithms.
https://en.wikipedia.org/wiki/Simplex_noise
Ultimately, they all follow a similar approach, so if we understand Simplex/Perlin we should be able to map those ideas to OpenSimplex2S.

---

1. Transform/skew input coordinates. This essentially transforms the grid into a honeycomb pattern, making the distance between points equal.
2. The values in the coordinate (x,y,z) are sorted in increasing order, e.g. (y,x,z). Then, ...?
3. The vertex is hashed to determine the gradient.
4. The contribution of each vertex of the simplex is combined. The input coordinate is subtracted to unskew the vector. The dot product then gives the gradient value. d^2 gives the squared distance to the point.