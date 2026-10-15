# Terrain Generation

Rather than using the old-school Perlin noise algorithm, I'll opt for the OpenSimplex2 algorithm.
In particular, I select the OpenSimplex2S variant (https://github.com/KdotJPG/OpenSimplex2/blob/master/java/OpenSimplex2S.java).
Instead of evaluating the BCC lattice precisely, it applies a geometric "skewing" factor to the coordinates before processing them.
This produces slightly smoother results, but is much faster.
Once we combine multiple noise layers, and since our world is blocky anyway, this difference should not be visible.
