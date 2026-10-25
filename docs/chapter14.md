# Serialization

We're very close to being able to spawn structures in our world.
The phased chunk generation was an important step, but there is one more thing we need before we can start adding structures: serialization.
Serialization is the process of converting internal Rust data into a format that can be saved to disk, and deserialization is the reverse.
We actually already did this before for the telemetry server, using JSON to send metrics over the network.

Why do we need serialization?
We don't have to; we could just make a function that spawn a tree at a given position.
However, as soon as we want some variation, perhaps some other structures, and a convenient way to observe and modify structures, we'll want them in a more convenient format.
Rather than making a temporary function that we have to replace later, we're better off doing it properly now.

That is not to say you should never take the easy approach, but you should be aware when none of the work you have done will be reusable later.
We have lots of temporary and partially complete systems, but they are all naturally evolving.
It's a trade-off you'll have to make for every feature, you'll almost never be able to immediately implement an intended feature in its entirety because so much depends on so much else.
