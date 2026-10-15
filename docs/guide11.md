# Performance debugging

At this point, I would like to see some performance statistics.
What is my framerate? How many chunks are currently loaded? What is my memory usage? etc.
However, I don't want to put all of this in the game view.

![](https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fgamepedia.cursecdn.com%2Fminecraft_gamepedia%2F1%2F18%2FDebugScreen.png&f=1&nofb=1&ipt=c6355da97e9a77c3a9d119ca871bf3f97922691cd487a843e0e9c61cad8e323d&ipo=images)

I find this very hard to read.
Rather, some plots and interactivity would be much nicer.

What we can do is to a browser-based debug dashboard.
- Bevy exposes selected metrics and debug settings to a WebSocket.
- A browser dashboard displays live graphs, counters, etc.

https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs
https://docs.rs/axum/latest/axum/extract/ws/

```bash
cargo add axum --features ws
cargo add tokio --features rt-multi-thread,macros,net,sync,time
cargo add serde --features derive
cargo add serde_json
```

- `axum` is a web framework for Rust, which we can use to serve a dashboard.
- `tokio` is an async runtime for Rust, which we need to run the web server outside the Bevy game loop. Otherwise, the web server would block the game loop.
- `serde` and `serde_json` are used to serialize the metrics data to JSON, which can be easily consumed by the browser dashboard.
