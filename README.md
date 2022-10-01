# axum-chat-example-rs

It's an example for Rust websocket chat application using [axum](https://github.com/tokio-rs/axum) 
and [dragonfly](https://github.com/dragonflydb/dragonfly) for Pub/Sub.

## Dependency

- [Rust with Cargo](http://rust-lang.org)
  - There is no specific `MSRV(Minimum Supported Rust Version)`
  - Only tested with the latest stable version Rust compiler (older/nightly builds may work...)
- docker tools (or you can install and start dragonfly server without docker.)

## How to start

```bash
% git clone git@github.com:kumanote/axum-chat-example-rs.git
% cd axum-chat-example-rs
# start dragonfly server by docker-compose
% docker-compose up
% cargo build --release
% ./target/release/axum-chat-example-server -a 0.0.0.0:3000
```

## Features

- Websocket chat using [dragonfly](https://github.com/dragonflydb/dragonfly) PUB/SUB backend.
- Works with multiple servers. (This may help horizontal scale.)

# References

- https://github.com/tokio-rs/axum/tree/main/examples/chat
