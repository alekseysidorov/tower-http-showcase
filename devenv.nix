{ config, pkgs, ... }:
let
  testWorkerPort = config.processes.test-worker.ports.http.value;
  testWorkerUrl = "http://127.0.0.1:${toString testWorkerPort}";
in
{
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  packages = [
    pkgs.curl
    pkgs.jq
  ];

  # Keep Cargo commands in one task so they share the build cache without
  # competing for Cargo's target-directory lock.
  tasks."checks:rust" = {
    description = "Run formatting, Clippy, and workspace checks";
    exec = ''
      cargo fmt --all -- --check
      cargo clippy --workspace --all-targets -- -D warnings
      cargo check --workspace
      cargo build -p showcase-server -p showcase-renderer
    '';
  };

  # The tests task namespace composes checks, Cargo tests, and HTTP smoke tests.
  # The full workspace test includes unit, Wiremock integration, and doc tests.
  tasks."tests:rust" = {
    description = "Run all workspace unit, integration, and doc tests";
    after = [ "checks:rust@succeeded" ];
    exec = "cargo test --workspace";
  };

  # Start the worker only after Cargo checks finish, so it cannot contend for
  # the target-directory lock while the Rust tasks are running.
  processes.test-worker = {
    after = [ "tests:rust@succeeded" ];
    exec = "exec target/debug/showcase-server";
    env = {
      LISTEN_ADDR = "127.0.0.1:${toString testWorkerPort}";
      WORKER_ID = "devenv-test-worker";
    };
    # Allocate instead of hard-coding a port so parallel checkouts do not clash.
    ports.http.allocate = 18081;
    ready = {
      http.get = {
        host = "127.0.0.1";
        port = testWorkerPort;
        path = "/health";
      };
      period = 1;
      timeout = 60;
    };
  };

  tasks."tests:worker-http" = {
    description = "Exercise a render request against the real Tokio worker";
    after = [ "devenv:processes:test-worker@ready" ];
    exec = ''
      set -euo pipefail
      response="$(curl --fail --silent --show-error \
        --request POST ${testWorkerUrl}/render \
        --header 'content-type: application/json' \
        --data '{"tile_id":7,"region":{"min_re":0,"max_re":0,"min_im":0,"max_im":0},"width":1,"height":1,"max_iterations":32}')"

      printf '%s\n' "$response" | jq --exit-status \
        --arg worker "devenv-test-worker" \
        '.tile_id == 7 and .worker_id == $worker and .runtime == "tokio" and .width == 1 and .height == 1 and .pixels == [0, 0, 0]' \
        >/dev/null
    '';
  };

  tasks."tests:renderer-http" = {
    description = "Render and verify a PNG through the real Tokio worker";
    after = [ "devenv:processes:test-worker@ready" ];
    exec = ''
      output=target/renderer-smoke.png
      cargo run --quiet -p showcase-renderer -- \
        --worker ${testWorkerUrl} \
        --width 3 --height 2 --tile-size 1 --max-iterations 32 \
        --output "$output"

      test "$(od -An -tx1 -N8 "$output" | tr -d '[:space:]')" = "89504e470d0a1a0a"
    '';
  };
}
