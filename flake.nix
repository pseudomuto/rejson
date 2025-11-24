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
        rustToolchain = pkgs.rust-bin.stable."1.91.1".minimal.override {
          extensions = [ "clippy" "llvm-tools-preview" ];
        };

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
      }
    );
}
