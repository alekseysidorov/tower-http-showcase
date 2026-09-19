# tower-http-showcase

A small distributed Mandelbrot rendering and tracing lab. It includes a
deterministic tile renderer, a Tokio worker, and a renderer that assembles a PNG
from tiles returned by one worker. Later milestones will connect rendering to
the existing Tower load balancer, add Compio, and export distributed traces.

## Run a Tokio worker

```sh
devenv shell cargo run -p showcase-server
```

The worker listens on `0.0.0.0:8080` by default. Configure it with
`WORKER_ID`, `LISTEN_ADDR`, and `WORKER_DELAY_MS`.

Render one RGB8 tile:

```sh
curl http://127.0.0.1:8080/render \
  -H 'content-type: application/json' \
  -d '{"tile_id":0,"region":{"min_re":-2.0,"max_re":1.0,"min_im":-1.0,"max_im":1.0},"width":128,"height":128,"max_iterations":500}'
```

The response contains the tile metadata and row-major RGB8 pixel bytes as a
JSON array.

## Render a PNG through one worker

The renderer splits an image into tiles, sends them concurrently to one worker,
and assembles the responses by tile coordinates before writing the PNG. The
default output is `target/mandelbrot.png`.

```sh
devenv shell cargo run -p showcase-renderer -- \
  --worker http://127.0.0.1:8080 \
  --width 1024 --height 1024 --tile-size 128 \
  --max-iterations 500 --output target/mandelbrot.png
```

## Tests

Run the complete configured check and test graph with:

```sh
devenv tasks run tests
```

This task graph runs formatting, Clippy, workspace checks and tests (including
Wiremock), then starts the test worker, waits for its `/health` probe, and
exercises both its HTTP API and PNG renderer. Devenv stops the worker when the
task graph finishes. To run just the renderer integration task and its dependencies:

```sh
devenv tasks run tests:renderer-http
```

Format Rust code with the shared workspace style using:

```sh
devenv tasks run format:rust
```
