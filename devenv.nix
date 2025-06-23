{ pkgs, lib, ... }:
{
  packages = with pkgs; [
    cargo-nextest
    git
    jq
    pkg-config
    openssl.dev
  ] ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk; [
    frameworks.Security
  ]);

  languages.rust = {
    enable = true;
    channel = "stable";
  };

  enterShell = "";

  env.RUST_LOG = "info";
  env.SHOWCASE_SERVER_PORT = "8080";

  scripts = {
    lints.exec = "cargo clippy";
  };

  processes = {
    showcase-server.exec = "cargo run --bin showcase-server";
    jaeger.exec = ''
      docker run --rm --name jaeger \
        -p 16686:16686 \
        -p 4317:4317 \
        -p 4318:4318 \
        -p 5778:5778 \
        -p 9411:9411 \
        jaegertracing/jaeger:2.7.0
    '';
  };

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # https://devenv.sh/tests/
  enterTest = ''
    wait_for_port $SHOWCASE_SERVER_PORT
    cargo run -p showcase-client --example check_server
  '';

  git-hooks.hooks = {
    clippy.enable = true;
    markdownlint.enable = true;
    nixpkgs-fmt.enable = true;
    rustfmt.enable = true;
    shellcheck.enable = true;
    taplo.enable = true;
  };
}
