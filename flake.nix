{
  description = "Microscopic fetch script in Rust, for NixOS systems";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];
    forEachSystem = nixpkgs.lib.genAttrs systems;
    pkgsForEach = system: nixpkgs.legacyPackages.${system};
  in {
    packages = forEachSystem (system: let
      pkgs = pkgsForEach system;
    in {
      default = self.packages.${system}.microfetch;
      microfetch = pkgs.callPackage ./nix/package.nix {};
    });

    devShells = forEachSystem (system: {
      default = (pkgsForEach system).callPackage ./nix/shell.nix {};
    });
  };
}
