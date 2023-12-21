{
  description = "Flake for lcmodmanager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, naersk, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          rust-overlay.overlays.default
          (final: prev: {
            rustToolchain =
              let
                rust = final.rust-bin;
              in
                rust.stable.latest.default.override {
                  extensions = [ "rust-src" ];
                  targets = [ "x86_64-unknown-linux-gnu" ];
                };
          })
        ];

        pkgs = import nixpkgs  {
          inherit system overlays;
          config.allowUnfree = true;
        };

        naersk' = pkgs.callPackage naersk {
          rustc = pkgs.rustToolchain;
          cargo = pkgs.rustToolchain;
        };

      in 
      {
        packages.x86_64-unknown-linux-gnu = naersk'.buildPackage {
          src = ./.;
          version = "0.1.0";
          name = "lcmodmanager";
          release = true;
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
          ];
        };

        devShells.default = pkgs.mkShell {
          allowUnfree = true;
          buildInputs = with pkgs; [
            vscode.fhs
            rustToolchain
            pkg-config
            openssl
            llvm_12
            lldb_12
          ];
        };
      });
}