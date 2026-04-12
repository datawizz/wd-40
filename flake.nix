{
  description = "WD-40 - Project Artifact Cleaner";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "wd-40";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          meta = with pkgs.lib; {
            description = "A CLI tool to recursively find and clean build artifacts";
            license = with licenses; [ mit asl20 ];
            mainProgram = "wd-40";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer

            # Node.js for integration tests
            nodejs_22

            # Python tooling
            uv

            # Additional utilities
            git
            bash
          ];

          shellHook = ''
            echo "🛢️  WD-40 Development Environment"
            echo ""
            echo "Available tools:"
            echo "  rustc:         $(rustc --version)"
            echo "  cargo:         $(cargo --version)"
            echo "  node:          $(node --version)"
            echo "  npm:           $(npm --version)"
            echo "  uv:            $(uv --version)"
            echo ""
            echo "Run 'cargo test' to run integration tests"
            echo "Run 'cargo build' to build the project"
          '';

          # Environment variables
          RUST_BACKTRACE = "1";
        };
      }
    );
}
