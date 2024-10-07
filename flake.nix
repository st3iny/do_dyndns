{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/release-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    utils,
    nixpkgs,
    rust-overlay,
    naersk,
  }:
  utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };
      naersk' = pkgs.callPackage naersk {};
    in rec {
      # `nix build` & `nix run`
      defaultPackage = naersk'.buildPackage {
        src = ./.;
      };

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rust-bin.nightly.latest.default
        ];
      };
    });
}
