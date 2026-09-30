{
  description = "Rejson - A utility for managing a collection of secrets in source control";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };

        # Use minimal stable Rust toolchain with clippy
        rustToolchain = pkgs.rust-bin.stable."1.98.1".minimal.override {
          extensions = [ "clippy" "llvm-tools-preview" ];
        };

        # MSRV toolchain, read from Cargo.toml so there's a single source of truth. rust-overlay
        # only has full versions (e.g. "1.90.0"), so pad a "1.90"-style rust-version.
        msrv =
          let
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.rust-version;
            parts = pkgs.lib.splitString "." version;
          in
          if builtins.length parts == 2 then "${version}.0" else version;

        msrvToolchain = pkgs.rust-bin.stable.${msrv}.minimal;

        # Use nightly rustfmt for formatting
        nightlyRustfmt = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.minimal.override {
            extensions = [ "rustfmt" ];
          }
        );
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            nightlyRustfmt

            # Additional development tools
            go-task
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          shellHook = ''
            echo "🦀 Rejson development environment loaded!"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo "Clippy available: $(clippy-driver --version)"
            echo "Rustfmt (nightly) available: $(rustfmt --version)"
          '';
        };

        # Checks that the code builds on the MSRV. Used by `task check:msrv`.
        devShells.msrv = pkgs.mkShell {
          nativeBuildInputs = [ msrvToolchain ];
        };
      }
    );
}
