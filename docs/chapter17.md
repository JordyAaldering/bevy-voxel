# Pump up the noise

There's lots of noise algorithms and additions, and lots of ways to use them.

- Layer noise for terrain generation
- Use noise for unique terrain features
- Caves, rivers
- Biome generation
- Voronoi noise for generating ore clusters?

The order matters.

## Caves

Caves can be generated almost separately, though it has to be before we place structures so we don't place trees in the air when a cave later carves out the ground below it.
Similarly for rivers.
For caves we can use the 3D simplex noise algorithm.

## Biomes

## Ores

Ores might depend on the biome, so they come after.

## Trees and other structures

Based on the biome, randomly place trees and other structures.
We also don't want for example ores to be places inside underground structures, so structures also need to come after them.
