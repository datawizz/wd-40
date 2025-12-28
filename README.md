# WD-40 🛢️

> *Penetrating deep into your filesystem to clean up stubborn build artifacts*

## What is WD-40?

`wd-40` is a command-line tool that recursively walks through directories and cleans build artifacts from Rust, Node.js, Python, and Haskell Stack projects, including compilation caches like sccache. Just like its namesake penetrating oil that helps remove rust and grime from metal, this tool removes build artifacts and cached dependencies from your filesystem.

## Why the name?

The name is a playful pun on multiple levels:
- **WD-40** is famous for cleaning and removing rust (the oxidation)
- This tool cleans **Rust** (the programming language) projects, plus Node.js, Python, and Haskell
- Just like WD-40 penetrates stuck parts, this tool penetrates deep into your directory structure
- WD-40 displaces unwanted buildup - this tool displaces unwanted build artifacts

*"When you've got too much buildup, reach for WD-40!"*

## Installation

```bash
cargo install wd-40
```

## Usage

Navigate to any directory and spray some WD-40 on it:

```bash
# Clean all artifacts (Rust, Node.js, Python) in current directory and subdirectories
wd-40

# Specify a target directory
wd-40 /path/to/workspace

# Dry run - see what would be cleaned without actually cleaning
wd-40 --dry-run

# Verbose output - see every artifact being cleaned
wd-40 -v

# Clean only Rust projects
wd-40 --rust-only

# Clean only Node.js node_modules
wd-40 --node-only

# Clean only Python virtual environments
wd-40 --python-only

# Clean only Haskell Stack projects
wd-40 --haskell-only

# Skip confirmation prompt
wd-40 -y
```

## What it does

`wd-40` will:
1. Recursively search for build artifacts in the specified directory:
   - **Rust projects**: Directories with `Cargo.toml` files
   - **Node.js projects**: `node_modules` directories with proper validation
   - **Python projects**: Virtual environments (`.venv`, `venv`, etc.)
   - **Haskell Stack projects**: Stack work directories (`.stack-work`)
   - **sccache directories**: Compilation cache directories (`.sccache`)
2. Delete the artifacts with robust validation to prevent false positives
3. Report how much disk space was freed
4. Show a detailed summary of all artifacts cleaned
5. Generate a log file in `~/.cache/wd-40/` for audit purposes

## Example Output

```
🛢️  WD-40 - Project Artifact Cleaner

Found 4 Rust projects
Found 3 node_modules directories
Found 2 Python virtual environments
Found 2 Stack work directories
Found 1 sccache directory

Proceed with cleaning? (y/N) y

✓ ~/projects/my-web-app
✓ ~/projects/cli-tool
✓ ~/projects/experiments/async-test
✓ ~/projects/experiments/macro-magic
📦 ~/projects/website/node_modules
📦 ~/projects/api/node_modules
📦 ~/projects/frontend/node_modules
🐍 ~/projects/ml-project/.venv
🐍 ~/projects/data-analysis/venv
λ ~/projects/haskell-parser/.stack-work
λ ~/projects/haskell-server/.stack-work
🔧 ~/workspace/.sccache

Summary:
         4 Rust projects cleaned
         3 node_modules
         2 Python venvs
         2 Stack work directories
         1 sccache directory
         4.15 GB total space freed

Log file: ~/.cache/wd-40/clean-20250112-143055.log
```

## Warning

**⚠️ This tool will delete all build artifacts!** This includes:

**Rust:**
- `target/` directories
- Compiled binaries
- Dependency caches
- Incremental compilation data

**Node.js:**
- `node_modules/` directories
- All installed npm/yarn/pnpm packages

**Python:**
- Virtual environment directories (`.venv`, `venv`, etc.)
- All installed Python packages in the venv

**Haskell Stack:**
- `.stack-work/` directories
- Compiled binaries and libraries
- Build artifacts and caches
- Stack SQLite databases

**sccache:**
- `.sccache/` directories
- Cached compilation objects
- All cached build artifacts

Make sure you actually want to clean these artifacts before running. Use `--dry-run` first if you're unsure.

### Safety Features

WD-40 includes multiple layers of validation to prevent false positives:

- **Rust targets**: Validates with `CACHEDIR.TAG` or `.rustc_info.json` markers
- **node_modules**: Requires parent directory to have `package.json`, `package-lock.json`, `yarn.lock`, or `pnpm-lock.yaml`
- **Python venvs**: Requires `pyvenv.cfg` file AND activation scripts AND lib directories
- **Stack work**: Validates `stack.sqlite3` OR `dist`/`install` directories AND parent has `stack.yaml`/`.cabal` file
- **sccache**: Validates directory name AND cache structure (subdirectories/files) AND excludes project directories

## Why use this?

- **Free up disk space**: Build artifacts and dependencies can consume massive amounts of space:
  - Rust's `target` directories (often 500MB-2GB per project)
  - Node.js `node_modules` (often 200MB-1GB per project)
  - Python virtual environments (often 100MB-500MB per venv)
  - Haskell Stack's `.stack-work` directories (often 200MB-1GB per project)
  - sccache compilation caches (often 1GB-10GB per cache)
- **Clean slate builds**: Sometimes you need to start fresh across multiple projects
- **Pre-backup cleanup**: Remove build artifacts before backing up source code
- **CI/CD pipelines**: Clean workspace between builds
- **System maintenance**: Quickly reclaim space when your disk is getting full

## Configuration

You can create a `.wd40ignore` file in any directory to prevent cleaning of specific projects:

```
# .wd40ignore
my-critical-project/
vendor/
**/node_modules/  # Just in case you have mixed workspaces
```

## Contributing

Found a stubborn bit of Rust that WD-40 can't clean? Open an issue!

PRs welcome - let's make this the ultimate Rust cleaning solution.

## License

MIT - Spray liberally and freely!

## Disclaimer

Not affiliated with the WD-40 Company. This tool won't actually help with squeaky hinges or stuck bolts. For actual rust (oxidation) problems, please use actual WD-40.

**Note**: This tool deletes directories. While it has extensive safety checks, always review what will be cleaned with `--dry-run` first. The author is not responsible for any data loss.
