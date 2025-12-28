#!/usr/bin/env bash
set -e

# This script creates test artifacts for WD-40 integration tests
# Usage: ./setup_test_artifacts.sh <test_directory>

TEST_DIR="${1:?Test directory path required}"

echo "Setting up test artifacts in: $TEST_DIR"

# Create the test directory
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "Creating Rust projects..."

# Create Rust Project 1 - Simple hello world
mkdir -p rust-project-1/src
cat > rust-project-1/Cargo.toml << 'EOF'
[package]
name = "rust-project-1"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF

cat > rust-project-1/src/main.rs << 'EOF'
fn main() {
    println!("Hello from rust-project-1!");
}
EOF

# Build the project to create target directory with proper markers
(cd rust-project-1 && cargo build --quiet 2>/dev/null || true)

# Create Rust Project 2 - With a dependency
mkdir -p rust-project-2/src
cat > rust-project-2/Cargo.toml << 'EOF'
[package]
name = "rust-project-2"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
EOF

cat > rust-project-2/src/main.rs << 'EOF'
fn main() {
    println!("Hello from rust-project-2!");
}
EOF

# Build the project to create target directory
(cd rust-project-2 && cargo build --quiet 2>/dev/null || true)

# Create Rust Project 3 - Simple lib
mkdir -p rust-project-3/src
cat > rust-project-3/Cargo.toml << 'EOF'
[package]
name = "rust-project-3"
version = "0.1.0"
edition = "2021"
EOF

cat > rust-project-3/src/lib.rs << 'EOF'
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
EOF

# Build the project
(cd rust-project-3 && cargo build --quiet 2>/dev/null || true)

echo "Creating orphaned target directory..."

# Create an orphaned target directory (no parent Cargo.toml)
mkdir -p orphaned-workspace/target
# Add Rust target markers to make it look like a real target directory
echo "Signature: 8a477f597d28d172789f06886806bc55" > orphaned-workspace/target/CACHEDIR.TAG
echo '{"rustc_fingerprint":12345}' > orphaned-workspace/target/.rustc_info.json
mkdir -p orphaned-workspace/target/debug

echo "Creating Node.js projects..."

# Create Node.js Project 1
mkdir -p node-project-1
cat > node-project-1/package.json << 'EOF'
{
  "name": "node-project-1",
  "version": "1.0.0",
  "description": "Test Node.js project",
  "main": "index.js",
  "dependencies": {
    "express": "^4.18.0"
  }
}
EOF

cat > node-project-1/index.js << 'EOF'
console.log("Hello from node-project-1");
EOF

# Install dependencies to create node_modules
(cd node-project-1 && npm install --silent --no-audit --no-fund 2>/dev/null || true)

# Create Node.js Project 2
mkdir -p node-project-2
cat > node-project-2/package.json << 'EOF'
{
  "name": "node-project-2",
  "version": "1.0.0",
  "description": "Another test Node.js project",
  "main": "app.js",
  "dependencies": {
    "lodash": "^4.17.21"
  }
}
EOF

cat > node-project-2/app.js << 'EOF'
console.log("Hello from node-project-2");
EOF

# Install dependencies
(cd node-project-2 && npm install --silent --no-audit --no-fund 2>/dev/null || true)

echo "Creating Python projects..."

# Create Python Project 1
mkdir -p python-project-1
cat > python-project-1/requirements.txt << 'EOF'
requests==2.31.0
EOF

cat > python-project-1/main.py << 'EOF'
print("Hello from python-project-1")
EOF

# Create venv using uv
(cd python-project-1 && uv venv .venv 2>/dev/null || python3 -m venv .venv)
# Install some packages to make it realistic
(cd python-project-1 && .venv/bin/pip install -q requests 2>/dev/null || true)

# Create Python Project 2
mkdir -p python-project-2
cat > python-project-2/requirements.txt << 'EOF'
numpy==1.24.0
EOF

cat > python-project-2/script.py << 'EOF'
print("Hello from python-project-2")
EOF

# Create venv using uv (or fallback to python3 -m venv)
(cd python-project-2 && uv venv .venv 2>/dev/null || python3 -m venv .venv)
# Install packages
(cd python-project-2 && .venv/bin/pip install -q numpy 2>/dev/null || true)

echo "Creating sccache directories..."

# Create sccache directory 1 - in a project directory
mkdir -p workspace/.sccache
# Add some cache-like structure
mkdir -p workspace/.sccache/tmp
mkdir -p workspace/.sccache/cache
echo "cached-object-1" > workspace/.sccache/cache/object1.o
echo "cached-object-2" > workspace/.sccache/cache/object2.o

# Create sccache directory 2 - standalone
mkdir -p build-cache/.sccache
mkdir -p build-cache/.sccache/objects
echo "build-artifact-1" > build-cache/.sccache/objects/artifact1.bin
echo "build-artifact-2" > build-cache/.sccache/objects/artifact2.bin

echo "Creating Haskell Stack projects..."

# Create Haskell Stack Project 1
mkdir -p haskell-project-1
cat > haskell-project-1/stack.yaml << 'EOF'
resolver: lts-21.0
packages:
- .
EOF

cat > haskell-project-1/package.yaml << 'EOF'
name: haskell-project-1
version: 0.1.0.0
dependencies:
- base >= 4.7 && < 5
executables:
  haskell-project-1-exe:
    main: Main.hs
    source-dirs: app
EOF

mkdir -p haskell-project-1/app
cat > haskell-project-1/app/Main.hs << 'EOF'
module Main where

main :: IO ()
main = putStrLn "Hello from haskell-project-1"
EOF

# Create a realistic .stack-work directory structure
mkdir -p haskell-project-1/.stack-work
mkdir -p haskell-project-1/.stack-work/dist
mkdir -p haskell-project-1/.stack-work/install
mkdir -p haskell-project-1/.stack-work/install/bin
mkdir -p haskell-project-1/.stack-work/install/doc
# Create the SQLite database file (characteristic marker)
echo "SQLite format 3" > haskell-project-1/.stack-work/stack.sqlite3
echo "lock" > haskell-project-1/.stack-work/stack.sqlite3.pantry-write-lock
# Add some build artifacts
mkdir -p haskell-project-1/.stack-work/dist/x86_64-linux/Cabal-3.8.1.0/build
echo "build-artifact" > haskell-project-1/.stack-work/dist/x86_64-linux/Cabal-3.8.1.0/build/artifact.o

# Create Haskell Stack Project 2
mkdir -p haskell-project-2
cat > haskell-project-2/stack.yaml << 'EOF'
resolver: lts-20.26
packages:
- .
EOF

cat > haskell-project-2/haskell-project-2.cabal << 'EOF'
cabal-version: 1.12
name: haskell-project-2
version: 0.1.0.0
build-type: Simple
library
  exposed-modules:
      Lib
  hs-source-dirs:
      src
  build-depends:
      base >=4.7 && <5
  default-language: Haskell2010
EOF

mkdir -p haskell-project-2/src
cat > haskell-project-2/src/Lib.hs << 'EOF'
module Lib (someFunc) where

someFunc :: IO ()
someFunc = putStrLn "Hello from haskell-project-2"
EOF

# Create .stack-work directory with different structure
mkdir -p haskell-project-2/.stack-work
mkdir -p haskell-project-2/.stack-work/dist
mkdir -p haskell-project-2/.stack-work/install/x86_64-osx/lts-20.26/9.2.8
mkdir -p haskell-project-2/.stack-work/install/x86_64-osx/lts-20.26/9.2.8/bin
# Create the SQLite database
echo "SQLite format 3" > haskell-project-2/.stack-work/stack.sqlite3
echo "lock" > haskell-project-2/.stack-work/stack.sqlite3.pantry-write-lock
# Add build artifacts
echo "library-artifact" > haskell-project-2/.stack-work/dist/lib.a

echo "Creating rustup directories..."

# Create rustup directory 1 - with full structure
mkdir -p home-sim/.rustup
cat > home-sim/.rustup/settings.toml << 'EOF'
default_host_triple = "x86_64-unknown-linux-gnu"
default_toolchain = "stable"
profile = "default"
version = "12"
EOF

mkdir -p home-sim/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin
echo "#!/bin/bash" > home-sim/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc
chmod +x home-sim/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc
mkdir -p home-sim/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib
touch home-sim/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/libstd.so

mkdir -p home-sim/.rustup/downloads
touch home-sim/.rustup/downloads/rust-1.70.0-x86_64-unknown-linux-gnu.tar.xz

mkdir -p home-sim/.rustup/update-hashes
echo "abc123" > home-sim/.rustup/update-hashes/stable-x86_64-unknown-linux-gnu

mkdir -p home-sim/.rustup/tmp

# Create rustup directory 2 - minimal structure
mkdir -p home-sim-2/.rustup
cat > home-sim-2/.rustup/settings.toml << 'EOF'
default_host_triple = "aarch64-apple-darwin"
default_toolchain = "nightly"
profile = "minimal"
version = "12"
EOF

mkdir -p home-sim-2/.rustup/toolchains
touch home-sim-2/.rustup/toolchains/.keep

echo "Creating Next.js projects..."

# Create Next.js Project 1 - with next.config.js
mkdir -p nextjs-project-1
cat > nextjs-project-1/package.json << 'EOF'
{
  "name": "nextjs-project-1",
  "version": "1.0.0",
  "dependencies": {
    "next": "14.0.0",
    "react": "18.2.0"
  }
}
EOF

cat > nextjs-project-1/next.config.js << 'EOF'
module.exports = {
  reactStrictMode: true,
}
EOF

# Create .next directory with realistic structure
mkdir -p nextjs-project-1/.next/cache
mkdir -p nextjs-project-1/.next/server
mkdir -p nextjs-project-1/.next/static
echo "build-id-12345" > nextjs-project-1/.next/BUILD_ID
echo "cached-build" > nextjs-project-1/.next/cache/build.json

# Create Next.js Project 2 - with next.config.mjs
mkdir -p nextjs-project-2
cat > nextjs-project-2/package.json << 'EOF'
{
  "name": "nextjs-project-2",
  "version": "1.0.0",
  "dependencies": {
    "next": "13.0.0"
  }
}
EOF

cat > nextjs-project-2/next.config.mjs << 'EOF'
export default {
  reactStrictMode: true,
}
EOF

mkdir -p nextjs-project-2/.next
echo "build-id-67890" > nextjs-project-2/.next/BUILD_ID
mkdir -p nextjs-project-2/.next/cache
echo "webpack-cache" > nextjs-project-2/.next/cache/webpack.json

echo "Creating cargo-nix directories..."

# Create cargo-nix directory 1 - with realistic structure
mkdir -p rust-nix-project-1
cat > rust-nix-project-1/Cargo.toml << 'EOF'
[package]
name = "rust-nix-project-1"
version = "0.1.0"
edition = "2021"
EOF

mkdir -p rust-nix-project-1/src
echo 'fn main() { println!("Hello"); }' > rust-nix-project-1/src/main.rs

mkdir -p rust-nix-project-1/.cargo-nix/target
echo "cached-artifact" > rust-nix-project-1/.cargo-nix/target/artifact.o
echo "cached-deps" > rust-nix-project-1/.cargo-nix/deps.lock

# Create cargo-nix directory 2 - minimal structure
mkdir -p rust-nix-project-2/.cargo-nix
echo "cache-data" > rust-nix-project-2/.cargo-nix/cache.bin
echo "nix-store-path" > rust-nix-project-2/.cargo-nix/store-path

echo "Test artifacts setup complete!"
echo ""
echo "Created:"
echo "  - 3 Rust projects with target directories"
echo "  - 1 orphaned target directory"
echo "  - 2 Node.js projects with node_modules"
echo "  - 2 Python projects with .venv directories"
echo "  - 2 sccache directories"
echo "  - 2 Haskell Stack projects with .stack-work directories"
echo "  - 2 rustup directories"
echo "  - 2 Next.js projects with .next directories"
echo "  - 2 cargo-nix directories"
