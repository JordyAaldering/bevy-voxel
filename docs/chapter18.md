# Shader programming

We now have a bunch of compute-intensive noise algorithms.
We can offload these to the GPU using shaders to improve performance.
However, this is not easy.
If we use a blocking compute shader, computations will stall the game and cause stuttering.
Instead, we should use _async readback_: the ore generation phase waits for results to come in before continuing.
