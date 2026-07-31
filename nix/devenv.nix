{ pkgs, ... }:
{
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  git-hooks = {
    package = pkgs.prek;
    hooks = {
      nix-fmt = {
        enable = true;
        name = "nix fmt";
        entry = "nix fmt";
        always_run = true;
        pass_filenames = false;
        stages = [ "pre-commit" ];
      };
      nix-flake-check = {
        enable = true;
        name = "nix flake check";
        entry = "nix flake check";
        always_run = true;
        pass_filenames = false;
        stages = [ "pre-push" ];
      };
    };
  };
}
