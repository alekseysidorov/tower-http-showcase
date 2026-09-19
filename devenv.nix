{ pkgs, ... }:
{
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  packages = with pkgs; [
    tombi
    marksman
  ];

  scripts.lints.exec = ''
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
  '';
}
