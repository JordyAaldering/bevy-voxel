# Foreword

I like figuring out how things work under the hood.
When developing something, I often wonder how a system works, whether it could be done differently, or how I would implement it myself.
There's a lot of satisfaction in understanding something well enough to build it yourself.

This guide follows the same approach.
The goal isn't just to make a voxel game, but to explore the concepts behind it and learn how to figure things out yourself.
I'll sometimes go off on tangents about things I find interesting, so if that sounds like your kind of thing, stick around.
If you just want a voxel engine, use something like [bevy-voxel-world](https://crates.io/crates/bevy_voxel_world) instead.

These are deliberately called _chapters_, rather than _tutorials_.
I provide code snippets instead of complete implementations because I want you to understand how things work, not just copy a solution.
Copy-pasting code doesn't teach you how to debug it, adapt it to your needs, or continue when you encounter a problem the guide doesn't cover.
I'll also refer to external resources where they explain a concept better than I can.

This approach requires some effort on your part.
Try to understand the concepts that interest you, and do your own research when something doesn't make sense.
Don't assume something is correct just because it's written confidently.
This is particularly relevant with AI-generated content, which can present incorrect information just as convincingly as correct information.
I'll try to keep tangents separate from the main subject so you can skip the ones that don't interest you.

I'm no Bevy or game design professional, so don't expect everything here to be optimal.
There are often multiple ways to solve a problem, and what works best depends on the circumstances.
The code might be out of date, and some solutions might not follow established Bevy conventions.
Treat this guide as a starting point, and feel free to point out mistakes or improvements in a GitHub issue.
If you find a better solution that doesn't make the guide more complex, please share it.

Before starting, I recommend reading the [Bevy introduction](https://bevy.org/news/introducing-bevy).
It's a little outdated, but explains the core ideas well.
Pay particular attention to the Entity Component System (ECS) paradigm, as understanding it is crucial for working with Bevy.
You don't need to know everything up front; just get a feel for what's available.

With that out of the way, let's get started.
We'll build up a voxel game step by step, exploring the concepts along the way.
The aim is to leave you with more than a working project: a better understanding of how things work and how to figure out the things we haven't covered.
