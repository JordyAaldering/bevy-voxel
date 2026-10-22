# Foreword

There's already so many voxel tutorials in the world, so why make another one?
Honestly, this is mainly for myself; normally I would blaze through the coding.
By developing the code in a tutorial format, I force myself to 'stop and smell the roses'.
First, this means we more carefully think about how our code will evolve or what kind of functionality we might need later.
I wouldn't say you should future-proof code for functionality you _might_ need, though when you have an idea of the kinds of things you need to take into account in the future, you can at least structure your code around that.
Second, taking it slow means we can focus on some other things than purely generating a world of blocks.
In these guides we'll also be looking into creating a telemetry web-server, learning how simplex noise works, creating an editor, etc.
Sure, we could use a noise library, there's a lot of satisfaction in understanding something well enough to build it yourself.
That should be why you're here, because otherwise you could just use a voxel engine, use something like [bevy-voxel-world](https://crates.io/crates/bevy_voxel_world) instead.
Are those needed just for making a voxel world?
No, but if you want to take this any further they are definitely useful.

I'm no Bevy or game design professional, so don't expect everything here to optimal.
The code might be out of date, and some solutions might not follow established Bevy conventions.
Treat this guide as a starting point, and feel free to suggest improvements in a GitHub issue.
If you find this guide useful, please star the repository, so I know that _someone_ is reading this, and I should continue working on it.

Before starting, I recommend reading the [Bevy introduction](https://bevy.org/news/introducing-bevy).
It's a little outdated, but explains the core ideas well.
In particular, the _Entity Component System_ (ECS) is an important concept, as it lies at the core of everything Bevy does.

With that out of the way, let's get started.
We'll build up a voxel game step by step, exploring the concepts along the way.
The aim is not to leave you with a working project.
Rather, I deliberately won't give complete code snippets.
That means more effort is required from your side, but ultimately it should teach you actual understanding of your code.

> Note: This guide is still a hot piece of garbage. As a placeholder some text is AI generated (it's better than nothing), so be aware that information may be incorrect. Please star the repository if you find this guide useful, so I know to spend time on the writing.
