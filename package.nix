{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "wd-40";
  version = "0.1.0";

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  # Integration tests require python3, node, stack, etc. which are
  # not available in the Nix build sandbox. The test suite is designed
  # to run in a full development environment via `cargo test`.
  doCheck = false;

  meta = with lib; {
    description = "A CLI tool to recursively find and clean build artifacts";
    longDescription = ''
      WD-40 recursively walks directory trees to find and safely remove build
      artifacts from Rust (target/), Node.js (node_modules/), Python (venv/),
      Haskell (.stack-work/), Next.js (.next/), and other ecosystems. Supports
      dry-run mode, selective cleaning, and detailed logging.
    '';
    homepage = "https://github.com/datawizz/wd-40";
    license = with licenses; [ mit asl20 ];
    maintainers = [ ];
    mainProgram = "wd-40";
    platforms = platforms.unix;
  };
}
