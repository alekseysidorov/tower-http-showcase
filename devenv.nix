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

  packages = with pkgs; [
    curl
    jq
    tombi
    marksman
  ];

  scripts.lints.exec = ''
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
  '';

  # Keep the in-process tests separate from the real worker HTTP smoke test.
  tasks."tests:unit".exec = "cargo test --workspace --lib";
  tasks."tests:wiremock".exec = "cargo test -p showcase-client --test hello_client";

  processes.test-worker = {
    exec = "cargo run --quiet -p showcase-server";
    env = {
      LISTEN_ADDR = "127.0.0.1:${toString testWorkerPort}";
      WORKER_ID = "devenv-test-worker";
    };
    ports.http.allocate = 18081;
    start.enable = false;
    ready.http.get = {
      host = "127.0.0.1";
      port = testWorkerPort;
      path = "/health";
    };
  };

  tasks."tests:worker-http" = {
    after = [ "devenv:processes:test-worker@ready" ];
    exec = ''
      response="$(curl --fail --silent --show-error \
        --request POST ${testWorkerUrl}/render \
        --header 'content-type: application/json' \
        --data '{"tile_id":7,"region":{"min_re":0,"max_re":0,"min_im":0,"max_im":0},"width":1,"height":1,"max_iterations":32}')"

      jq --exit-status \
        --arg worker "devenv-test-worker" \
        '.tile_id == 7 and .worker_id == $worker and .runtime == "tokio" and .width == 1 and .height == 1 and .pixels == [0, 0, 0]' \
        <<< "$response" >/dev/null
    '';
  };
}
