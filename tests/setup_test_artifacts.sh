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

echo "Test artifacts setup complete!"
echo ""
echo "Created:"
echo "  - 3 Rust projects with target directories"
echo "  - 1 orphaned target directory"
echo "  - 2 Node.js projects with node_modules"
echo "  - 2 Python projects with .venv directories"
