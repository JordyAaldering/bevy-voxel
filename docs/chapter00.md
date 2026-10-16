# Foreword

These are deliberately named 'chapters', and not 'tutorials', as the purpose is to teach you how to do this yourself, rather than simply giving a copy-pastable solution.
To this end, I deliberately only show code snippets, rather than the entire code.
The purpose is for you to learn how to do this yourself.
By simply copy-pasting the code you don't learn how to fix bugs or problems in the code, how to adjust it to your liking, and ultimately how to continue after the chapter ends.
For that same reason I will often use external resources to explain certain concepts: if you want to actually understand what is going on it is crucial to look for explanations rather than blindly copying.

If you just want a Voxel engine, use something like [bevy-voxel-world](https://crates.io/crates/bevy_voxel_world) rather than copying code from here.

In particular, I want you to really try to understand what I write here.
Especially in a world with tons of AI-generated content, people are more and more conditioned to just blindly read a text, 'one ear in and one ear out' (or I guess eye in this case), and not really think about it.
(Deliberately put a sentence here that makes no sense).

If the previous sentence did not bother you, you failed the first test.
I encourage you to do your own research.
I am no Bevy or game design professional, so I am sure that I do things in a suboptimal way.
Anyway, everyone has their own development style.
If something is unclear, please let me know in a GitHub issue.
If you find out I've done something in a suboptimal way, (which can be rectified without making the guide more complex), please share.

Bevy states:

> Bevy is still in development. Important features are missing, and documentation can be sparse. A new version of Bevy containing breaking changes to the API is released approximately once every 3 months. We provide migration guides, but we can't guarantee that migrations will always be easy. Use only if you are willing to work in this environment.

And the same holds here: only use this guide if you are willing to put in the work.
I only provide code snippets, code might be out of date, and I'm sure I've done things in a suboptimal way.
It is up to you to decide where to put in extra work to improve your understanding, or the code.

The Bevy introduction is quite nice: https://bevy.org/news/introducing-bevy.
It is a little outdated, but brings the core ideas across.
Definitely read the initial parts about ECS, and just scroll through the rest to get an idea of what is available.
In particular, it is crucial that you understand the Entity Component System (ECS) paradigm.
