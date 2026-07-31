{
  description = "Tower http client showcase";

  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devenv.url = "github:cachix/devenv";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Adds the Cachix cache to the Nix configuration
  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      imports = [
        inputs.devenv.flakeModule
        inputs.treefmt-nix.flakeModule
      ];

      perSystem = { config, ... }: {
        treefmt = {
          projectRootFile = "flake.nix";

          programs = {
            nixfmt.enable = true;
            rustfmt = {
              enable = true;
              # package = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
            };
            beautysh.enable = true;
            deno.enable = true;
            taplo.enable = true;
          };
        };

        packages = { };
        checks = { };
        apps = { };

        devenv.shells.default = {
          imports = [ ./nix/devenv.nix ];
        };
      };
    };
}
