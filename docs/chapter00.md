# Foreword

I like figuring out how things work under the hood.
When developing something, I often run into things that I think can maybe be done differently, or wonder how a particular system works.
There is a lot of satisfaction in truly understanding how something works, and being able to implement it yourself.

This guide follows the same approach.
The goal is not simply to make a voxel game, but to understand the concepts behind it and learn how to figure things out yourself.
If you just want a voxel engine, use something like [bevy-voxel-world](https://crates.io/crates/bevy_voxel_world) instead.

These are deliberately named *chapters*, and not *tutorials*.
I want to teach you how to do things yourself, rather than give you a copy-pasteable solution.
That's why I only show code snippets instead of the entire code.
Copy-pasting code doesn't teach you how to fix bugs, adapt things to your needs, or continue after the chapter ends.
For the same reason, I often refer to external resources.
If you want to understand what's going on, you need to do more than just copy code.

In particular, I want you to actually think about what you read.
People often blindly consume information without thinking for a second about whether it _actually_ makes sense.
This is especially important now that AI-generated content is so common, which will blatantly lie to you and try to get away with it.
Ideally, you truly attempt to understand the concepts in these chapters (that is; the ones you find interesting, I'll try to develop these chapters in such a way that you can skip the tangents).

I encourage you to do your own research.
I'm no Bevy or game design professional, so I'm sure I do things in suboptimal ways.
Everyone has their own development style, and there is often more than one way to solve a problem.
If something is unclear, please let me know in a GitHub issue.
If you find a better solution that doesn't make the guide more complex, please share it.

Bevy itself comes with a warning:

> Bevy is still in development. Important features are missing, and documentation can be sparse. A new version of Bevy containing breaking changes to the API is released approximately once every 3 months. We provide migration guides, but we can't guarantee that migrations will always be easy. Use only if you are willing to work in this environment.

The same applies here.
The code might be out of date, and some solutions might not be optimal or the 'Bevy way' of doing things.
Use this guide as a starting point, and decide for yourself where to dig deeper.

Before starting, I recommend reading the [Bevy introduction](https://bevy.org/news/introducing-bevy).
It's a little outdated, but explains the core ideas well.
Pay particular attention to the Entity Component System (ECS) paradigm, as understanding it is crucial for working with Bevy.
You don't need to know everything up front; just get a feel for what's available.

With that out of the way, let's get started.
We'll build up the concepts behind a voxel game step by step, with the aim of understanding not just how things work, but how to figure them out yourself.
