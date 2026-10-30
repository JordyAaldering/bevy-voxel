# Pump up the noise

Each chunk should get its own random seed/state (for local randomness, like structures, terrain should remain consistent across chunks.), based on the original seed combined with the chunk's position.
This ensures that randomness is not dependent on the order in which chunks are generated.
We also need a per-chunk random state, because a global state would sequentialize generation.

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

### Steps performed

1. **Add a generation phase.** `ChunkGenerationPhase` gets a `CaveGeneration` phase between `HeightGeneration` and `GenerateStructures`, and `next()` is updated to match.
   Saved chunks store the phase by variant index, so chunks saved before this change are stale: delete `savedata/world/chunk_*.bin`.
2. **Declare the neighbors each phase needs.** Phases do not all need the same chunks to be ready, so each phase now says which ones, using `ChunkGenerationPhase::neighbor_requirement()`.
   It returns a `NeighborRequirement { radius, diagonal }`: how many chunks away to look (0 is only the chunk itself), and whether chunks outside the same row or column count.
   - `HeightGeneration` and `CaveGeneration` need nothing, because noise is a pure function of the world position, so the chunk only reads and writes itself.
   - `GenerateStructures` needs radius 1 including diagonals, because a tree near a corner can overflow into the diagonal chunk.
   - `MeshGeneration` needs radius 1 without diagonals, because only the touching chunks matter for hiding faces.
3. **Check readiness in one place.** The separate `neighbors_ready_for_structures` and `neighbors_ready_for_mesh` are replaced by `VoxelWorld::neighbors_ready_for(coord, phase)`, which checks every offset of the phase's requirement.
4. **Make 3D noise available.** `NoiseSettings::sample3` is the fractal version of the 3D OpenSimplex2S noise, with the same octaves, frequency, and so on as `sample2`.
5. **Add `Caves` to `WorldConfig`.**
   It holds the `noise` settings and a `threshold`.
   It is `#[serde(default)]`, so existing `world.json` files keep loading and get the default caves.
6. **Carve.** `generate_caves` visits every column of the chunk and sets a voxel to air when the 3D noise at its world position is above `threshold`.
   Because the noise only depends on the world position, caves continue across chunk borders without needing to look at the neighbors.
   Only the lowest layer is never carved, so caves do not open up the bottom of the world.
   Caves can break through the surface, which gives cave entrances.
   The noise has its own seed offset, so it does not correlate with the terrain.
7. **Test it.** One test checks that caves are a small part of the underground (the default is about 3%), and another that an old config without `caves` still loads.

### Tuning

- `threshold` decides how much is carved out: lower is more cave. Since the noise roughly spans `[-1, 1]`, `0.5` carves a few percent.
- `noise.frequency` decides the size of the caves, and `octaves` makes their walls rougher.
- Thresholding blobs gives round caverns. Tunnels need a different shape of noise, for example carving where the absolute value of the noise is close to zero.

## Biomes

Biomes change how the terrain looks: the height, the blocks on top, and how many trees grow.
The challenge is that the transition between two biomes must not be a cliff.

### Steps performed

1. **Pick an algorithm.** Biomes come from _climate noise_ rather than Voronoi cells.
   Two extra low-frequency noise maps (temperature and humidity, roughly `[-1, 1]`) are sampled per column, using the same OpenSimplex2S as the height, but with their own seed offsets so they are not correlated.
   Each biome has a target point in this temperature/humidity space, and the closest one wins.
   The biome is a pure function of `(x, z)` and the seed, so it is never stored: not in chunks, and not in the save format.
   Chunks still generate independently.
2. **Remove `TerrainConfig`.** The seed is now part of the world, and the height noise settings moved into each biome.
3. **Define `Biome`** (`src/biome.rs`).
   Besides the climate position (`temperature`, `humidity`), it holds `height: NoiseSettings`, `surface` and `subsurface` voxels, and `tree_density`.
   Add more fields (ores, structures) when they are needed, not before.
4. **Define `WorldConfig`.**
   It contains the `seed`, the `Climate` (both noise settings, and `blend`), and the `Vec<Biome>`.
   It is serialized with serde to `savedata/world/world.json` next to the chunks, so every world can have its own settings.
   `NoiseSettings` and `NoiseType` therefore derive `Serialize` and `Deserialize` (the struct with `#[serde(default)]`, so JSON only has to list what differs from the default).
   `WorldConfig::load_or_create` writes the defaults if there is no file yet, so there is something to edit.
   An invalid file is an error, instead of falling back to the defaults, because the terrain would no longer match the saved chunks.
5. **Create plains, desert, and mountains.**
   Plains are mild and grassy with some trees, deserts are flat and sandy without trees, and mountains are tall (more octaves, high amplitude and bias) with a stone surface.
6. **Blend the height.** `WorldConfig::column(x, z)` samples the climate and computes the distance of each biome to it.
   The weight of a biome is `max(0, nearest + blend - distance)`, so the nearest biome always counts, and biomes that are at most `blend` further away fade in linearly.
   The height is the weighted average of the height of each biome with a non-zero weight.
   Deep inside a biome only that biome is sampled, and at a border both are, which makes the height continuous.
   Every biome is sampled at the same world `(x, z)`, and the weights only depend on the position, so chunk borders show no seams either.
7. **Dither the blocks.** Voxels cannot be averaged, so each column picks one biome for its surface, subsurface, and tree density, with a probability proportional to the weights.
   The choice comes from a hash of the position (`hash01`), so it is deterministic, and independent of the order in which chunks generate.
   This makes the border between grass and sand grainy instead of a straight line.
8. **Use it in generation.** `generate_chunk_height` asks for the `Column` of each position and fills surface, two layers of subsurface (`SUBSURFACE_DEPTH`), and stone below.
   `generate_structures` now visits every column and places a tree if `has_tree` says so, replacing the two hardcoded tree columns per chunk.
   The tree is placed on the column's height, so it follows the terrain.
9. **Test it.** One test checks that the config survives a JSON round trip.
   The other walks over thousands of columns, checks that all three biomes occur, and that the height never changes more than a few blocks between neighbouring columns.

### Tuning

- `climate` frequency decides the size of biomes. It should be much lower than the height frequency, otherwise biomes are smaller than the hills.
- `blend` decides the width of the transition. `0` gives hard borders, and large values blend all biomes everywhere.
- The temperature/humidity points decide how common each biome is: the more space around a point is closest to it, the larger the biome.
- Existing chunks are saved, so delete `savedata/world/chunk_*.bin` to see a changed configuration.

## Ores

Ores might depend on the biome, so they come after.

## Trees and other structures

Based on the biome, randomly place trees and other structures.
We also don't want for example ores to be places inside underground structures, so structures also need to come after them.
