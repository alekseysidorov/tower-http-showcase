{ pkgs, lib, ... }:
{
  packages = with pkgs; [
    cargo-nextest
    git
    jq
    pkg-config
    openssl.dev
  ] ++ lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
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
    ci-lints.exec = "cargo clippy";
    ci-tests.exec = "cargo nextest run";
  };

  processes = {
    showcase-server.exec = "cargo run --bin showcase-server";
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
