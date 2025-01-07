# treefmt.nix
{ ... }:
{
  # Used to find the project root
  projectRootFile = "devenv.nix";

  programs.nixpkgs-fmt.enable = true;
  programs.rustfmt.enable = true;
  programs.beautysh.enable = true;
  programs.deno.enable = true;
  programs.taplo.enable = true;
}
