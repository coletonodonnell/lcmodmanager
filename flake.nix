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
                  targets = [ "x86_64-pc-windows-gnu" "x86_64-unknown-linux-gnu" ];
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

        packages.x86_64-pc-windows-gnu = naersk'.buildPackage {
          src = ./.;
          strictDeps = true;

          name = "lcmodmanager";

          depsBuildBuild = with pkgs; [
            pkgsCross.mingwW64.stdenv.cc
            # pkgsCross.mingwW64.windows.pthreads
          ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            boringssl
            openssl.dev
            openssl
            wineWowPackages.stable
          ];

          buildInputs = with pkgs; [
            openssl
            openssl.dev
            boringssl
          ];

          doCheck = true;

          # foo = builtins.trace ''${builtins.getEnv "NIX_LDFLAGS"}'' 1;

          # CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = ''
          #   -C link-args=${builtins.getEnv "NIX_LDFLAGS"} -L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib
          # '';

          # Tells Cargo that we're building for Windows.
          # (https://doc.rust-lang.org/cargo/reference/config.html#buildtarget)
          CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

          # Tells Cargo that it should use Wine to run tests.
          # (https://doc.rust-lang.org/cargo/reference/config.html#targettriplerunner)
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER = pkgs.writeScript "wine-wrapper" ''
            export WINEPREFIX="$(mktemp -d)"
            exec wine64 $@
          '';
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
            zlib
          ];
        };
      });
}