{ pkgs, ... }:

{
  packages = with pkgs; [
    cargo-nextest
    git
  ];

  languages.rust.enable = true;

  enterShell = "";

  scripts = {
    ci-lints.exec = "cargo clippy";
    ci-tests.exec = "cargo nextest run";
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
    shellcheck.enable = true;
  };
}
