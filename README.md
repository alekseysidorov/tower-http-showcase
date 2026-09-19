# tower-http-showcase

A small distributed Mandelbrot rendering and tracing lab. The repository
already contains a Tower client-side load-balancing demo; this first milestone
adds a deterministic, synchronous tile renderer and a Tokio worker. Later
milestones will connect tile rendering to the balancer, add Compio, and export
distributed traces.

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

## Tests

Unit tests and Wiremock-backed client tests run with:

```sh
devenv tasks run tests
```

The task namespace includes unit tests, Wiremock-backed HTTP client tests, and a
real Tokio worker smoke test that starts the server, waits for `/health`, then
posts a tile to `/render` and checks the response. Run only the real worker test
with:

```sh
devenv tasks run tests:worker-http
```
