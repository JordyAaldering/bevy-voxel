# Better error messages

Similar to the popular anyhow crate, `BevyError` now provides an ergonomic way to attach extra context to an error using the context method, which also allows creating a `Result<T, BevyError>` from an `Option<T>`.

This makes it easier to trace back errors with human-readable messages without looking at verbose backtraces.

https://bevy.org/news/bevy-0-20/#bevy-error-context-messages
