{ inputs, pkgs, ... }:
let
  treefmt-nix = inputs.treefmt-nix;
  treefmt = (treefmt-nix.lib.evalModule pkgs ./treefmt.nix).config.build;
in
{
  packages = with pkgs; [
    cargo-nextest
    git
  ];

  languages.rust.enable = true;

  enterShell = "";

  env.RUST_LOG = "info";

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
    echo "Hello world"
  '';

  pre-commit.hooks = {
    nixpkgs-fmt.enable = true;
    rustfmt.enable = true;
    clippy.enable = true;
    shellcheck.enable = true;
    treefmt = {
      enable = true;
      package = treefmt.wrapper;
    };
  };
}
