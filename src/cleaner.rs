use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub enum CleanStatus {
    Success { space_freed: Option<u64> },
    TargetOnly { space_freed: u64, reason: String },
    Failed(String),
    Skipped(String),
}

pub struct CleanResult {
    pub project_path: String,
    pub status: CleanStatus,
}

impl CleanResult {
    pub fn is_success(&self) -> bool {
        matches!(self.status, CleanStatus::Success { .. })
    }

    pub fn is_target_only(&self) -> bool {
        matches!(self.status, CleanStatus::TargetOnly { .. })
    }

    pub fn is_skipped(&self) -> bool {
        matches!(self.status, CleanStatus::Skipped(_))
    }

    pub fn space_freed(&self) -> Option<u64> {
        match &self.status {
            CleanStatus::Success { space_freed } => *space_freed,
            CleanStatus::TargetOnly { space_freed, .. } => Some(*space_freed),
            _ => None,
        }
    }
}

/// Calculates the total size of a directory recursively
pub fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    if !path.exists() {
        return Ok(0);
    }

    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            total_size += calculate_dir_size(&entry.path())?;
        }
    }

    Ok(total_size)
}

/// Validates if a directory is a Rust target directory by checking for Cargo-specific markers
pub fn is_rust_target_dir(path: &Path) -> bool {
    // Must be named exactly "target" or "target-ra" (rust-analyzer cache)
    let is_valid_name = path.file_name().and_then(|n| n.to_str()).map_or(false, |name| {
        name == "target" || name == "target-ra"
    });

    if !is_valid_name {
        return false;
    }

    // Must NOT contain a Cargo.toml (safety check - could be a project named "target")
    if path.join("Cargo.toml").exists() {
        return false;
    }

    // Must contain at least one Rust-specific marker
    let has_cachedir_tag = path.join("CACHEDIR.TAG").exists();
    let has_rustc_info = path.join(".rustc_info.json").exists();

    has_cachedir_tag || has_rustc_info
}

/// Validates if a directory is a node_modules directory by checking multiple attributes
pub fn is_node_modules_dir(path: &Path) -> bool {
    // Must be named "node_modules"
    if path.file_name().and_then(|n| n.to_str()) != Some("node_modules") {
        return false;
    }

    // Safety: Must NOT contain project markers that would indicate this is not a node_modules
    if path.join("Cargo.toml").exists() || path.join("setup.py").exists() {
        return false;
    }

    // Must have a parent directory
    let parent = match path.parent() {
        Some(p) => p,
        None => return false,
    };

    // Parent must contain at least one Node.js project marker
    let has_package_json = parent.join("package.json").exists();
    let has_package_lock = parent.join("package-lock.json").exists();
    let has_yarn_lock = parent.join("yarn.lock").exists();
    let has_pnpm_lock = parent.join("pnpm-lock.yaml").exists();

    if !has_package_json && !has_package_lock && !has_yarn_lock && !has_pnpm_lock {
        return false;
    }

    // Additional verification: check if directory has typical node_modules structure
    // Look for .bin directory or .package-lock.json or subdirectories with package.json
    let has_bin = path.join(".bin").exists();
    let has_package_lock_json = path.join(".package-lock.json").exists();

    // Check if it has subdirectories (node_modules typically contains packages)
    let has_subdirectories = fs::read_dir(path)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .find(|e| e.path().is_dir())
        })
        .is_some();

    has_bin || has_package_lock_json || has_subdirectories
}

/// Validates if a directory is a Python virtual environment by checking multiple attributes
pub fn is_python_venv_dir(path: &Path) -> bool {
    let dir_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    // Must be named one of the common venv names (but NOT .env to avoid env files)
    let valid_names = ["venv", ".venv", "env", "ENV", "virtualenv", ".virtualenv"];
    if !valid_names.contains(&dir_name) {
        return false;
    }

    // Safety: Must NOT contain git directory (avoid false positives with repos named venv)
    if path.join(".git").exists() {
        return false;
    }

    // Must contain pyvenv.cfg file (definitive marker for Python venvs)
    if !path.join("pyvenv.cfg").exists() {
        return false;
    }

    // Must have virtual environment structure (bin or Scripts for Windows)
    let has_bin = path.join("bin").exists();
    let has_scripts = path.join("Scripts").exists();

    if !has_bin && !has_scripts {
        return false;
    }

    // Check for activation scripts
    let has_activate_unix = path.join("bin").join("activate").exists();
    let has_activate_windows = path.join("Scripts").join("activate.bat").exists();

    if !has_activate_unix && !has_activate_windows {
        return false;
    }

    // Check for lib or Lib directory with Python
    let has_lib = path.join("lib").exists();
    let has_lib_windows = path.join("Lib").exists();

    has_lib || has_lib_windows
}

/// Validates if a directory is an sccache cache directory by checking multiple attributes
pub fn is_sccache_dir(path: &Path) -> bool {
    // Must be named ".sccache"
    if path.file_name().and_then(|n| n.to_str()) != Some(".sccache") {
        return false;
    }

    // Safety: Must NOT contain project markers (avoid false positives)
    if path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join(".git").exists() {
        return false;
    }

    // Check if directory has cache-like structure
    // sccache typically contains subdirectories with cached compilation artifacts
    let has_subdirectories = fs::read_dir(path)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .find(|e| e.path().is_dir())
        })
        .is_some();

    // Also check for files that would indicate this is a cache directory
    let has_files = fs::read_dir(path)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(|e| e.ok())
                .find(|e| e.path().is_file())
        })
        .is_some();

    // Must have either subdirectories or files (not be empty)
    has_subdirectories || has_files
}

/// Validates if a directory is a Haskell Stack work directory by checking multiple attributes
pub fn is_stack_work_dir(path: &Path) -> bool {
    // Must be named ".stack-work"
    if path.file_name().and_then(|n| n.to_str()) != Some(".stack-work") {
        return false;
    }

    // Safety: Must NOT contain project markers (avoid false positives)
    if path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join(".git").exists()
        || path.join("setup.py").exists() {
        return false;
    }

    // Check for Stack-specific markers
    // Stack work directories contain at least one of these characteristic structures
    let has_sqlite_db = path.join("stack.sqlite3").exists();
    let has_dist_dir = path.join("dist").exists();
    let has_install_dir = path.join("install").exists();

    // Must have at least one Stack marker
    if !has_sqlite_db && !has_dist_dir && !has_install_dir {
        return false;
    }

    // Additional validation: parent should ideally have stack.yaml or package.yaml (Stack project markers)
    // This is optional but adds extra safety
    if let Some(parent) = path.parent() {
        let has_stack_yaml = parent.join("stack.yaml").exists();
        let has_package_yaml = parent.join("package.yaml").exists();
        let has_cabal_file = fs::read_dir(parent)
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .find(|e| {
                        e.path().extension()
                            .and_then(|ext| ext.to_str()) == Some("cabal")
                    })
            })
            .is_some();

        // If parent exists, it should have at least one Haskell project marker
        // OR we should have strong Stack markers (sqlite db is very specific)
        if !has_stack_yaml && !has_package_yaml && !has_cabal_file && !has_sqlite_db {
            return false;
        }
    }

    true
}

/// Validates if a directory is a rustup installation directory by checking multiple attributes
pub fn is_rustup_dir(path: &Path) -> bool {
    // Must be named ".rustup"
    if path.file_name().and_then(|n| n.to_str()) != Some(".rustup") {
        return false;
    }

    // Safety: Must NOT contain project markers (avoid false positives)
    if path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join(".git").exists() {
        return false;
    }

    // Must contain at least one rustup-specific marker
    let has_settings = path.join("settings.toml").exists();
    let has_toolchains = path.join("toolchains").exists();
    let has_downloads = path.join("downloads").exists();
    let has_update_hashes = path.join("update-hashes").exists();

    // Must have at least one rustup marker
    has_settings || has_toolchains || has_downloads || has_update_hashes
}

/// Validates if a directory is a Next.js build directory by checking multiple attributes
pub fn is_next_dir(path: &Path) -> bool {
    // Must be named ".next"
    if path.file_name().and_then(|n| n.to_str()) != Some(".next") {
        return false;
    }

    // Safety: Must NOT contain project markers (avoid false positives)
    if path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join(".git").exists() {
        return false;
    }

    // Must contain at least one Next.js marker
    let has_build_id = path.join("BUILD_ID").exists();
    let has_cache = path.join("cache").exists();
    let has_server = path.join("server").exists();
    let has_static = path.join("static").exists();

    // Must have at least one Next.js marker
    if !has_build_id && !has_cache && !has_server && !has_static {
        return false;
    }

    // Parent should have Next.js project markers
    if let Some(parent) = path.parent() {
        let has_next_config_js = parent.join("next.config.js").exists();
        let has_next_config_mjs = parent.join("next.config.mjs").exists();
        let has_next_config_ts = parent.join("next.config.ts").exists();
        let has_package_json = parent.join("package.json").exists();

        return has_next_config_js || has_next_config_mjs || has_next_config_ts || has_package_json;
    }

    false
}

/// Validates if a directory is a cargo-nix cache directory by checking multiple attributes
pub fn is_cargo_nix_dir(path: &Path) -> bool {
    // Must be named ".cargo-nix"
    if path.file_name().and_then(|n| n.to_str()) != Some(".cargo-nix") {
        return false;
    }

    // Safety: Must NOT contain project markers (avoid false positives)
    if path.join("Cargo.toml").exists()
        || path.join("package.json").exists()
        || path.join(".git").exists() {
        return false;
    }

    // Must have content (files or subdirectories) - not empty
    let has_content = fs::read_dir(path)
        .ok()
        .and_then(|entries| entries.filter_map(|e| e.ok()).next())
        .is_some();

    has_content
}

/// Safely deletes a Rust target directory with multiple verification layers
pub fn delete_target_dir(target_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Triple verification
    if !is_rust_target_dir(target_path) {
        return Ok(None);
    }

    // Verify it's a direct sibling of a Cargo.toml
    if let Some(parent) = target_path.parent() {
        if !parent.join("Cargo.toml").exists() {
            return Ok(None);
        }
    } else {
        return Ok(None);
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(target_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(target_path)
        .with_context(|| format!("Failed to delete target directory: {}", target_path.display()))?;

    Ok(Some(size))
}

/// Safely deletes an orphaned Rust target directory (target without parent Cargo.toml)
pub fn delete_orphaned_target_dir(target_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Verify it's a Rust target directory
    if !is_rust_target_dir(target_path) {
        return Ok(None);
    }

    // Verify parent does NOT have Cargo.toml (confirming it's orphaned)
    if let Some(parent) = target_path.parent() {
        if parent.join("Cargo.toml").exists() {
            // Not orphaned - has a parent Cargo.toml
            return Ok(None);
        }
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(target_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(target_path)
        .with_context(|| format!("Failed to delete orphaned target directory: {}", target_path.display()))?;

    Ok(Some(size))
}

/// Safely deletes a node_modules directory with verification
pub fn delete_node_modules_dir(node_modules_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Verify it's actually a node_modules directory
    if !is_node_modules_dir(node_modules_path) {
        return Ok(None);
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(node_modules_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(node_modules_path)
        .with_context(|| format!("Failed to delete node_modules directory: {}", node_modules_path.display()))?;

    Ok(Some(size))
}

/// Safely deletes a Python virtual environment directory with verification
pub fn delete_venv_dir(venv_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Verify it's actually a Python venv
    if !is_python_venv_dir(venv_path) {
        return Ok(None);
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(venv_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(venv_path)
        .with_context(|| format!("Failed to delete Python venv directory: {}", venv_path.display()))?;

    Ok(Some(size))
}

/// Safely deletes an sccache cache directory with verification
pub fn delete_sccache_dir(sccache_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Verify it's actually an sccache directory
    if !is_sccache_dir(sccache_path) {
        return Ok(None);
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(sccache_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(sccache_path)
        .with_context(|| format!("Failed to delete sccache directory: {}", sccache_path.display()))?;

    Ok(Some(size))
}

/// Safely deletes a Haskell Stack work directory with verification
pub fn delete_stack_work_dir(stack_work_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Verify it's actually a Stack work directory
    if !is_stack_work_dir(stack_work_path) {
        return Ok(None);
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(stack_work_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(stack_work_path)
        .with_context(|| format!("Failed to delete Stack work directory: {}", stack_work_path.display()))?;

    Ok(Some(size))
}

/// Safely deletes a rustup installation directory with verification
pub fn delete_rustup_dir(rustup_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Verify it's actually a rustup directory
    if !is_rustup_dir(rustup_path) {
        return Ok(None);
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(rustup_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(rustup_path)
        .with_context(|| format!("Failed to delete rustup directory: {}", rustup_path.display()))?;

    Ok(Some(size))
}

/// Safely deletes a Next.js build directory with verification
pub fn delete_next_dir(next_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Verify it's actually a Next.js build directory
    if !is_next_dir(next_path) {
        return Ok(None);
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(next_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(next_path)
        .with_context(|| format!("Failed to delete .next directory: {}", next_path.display()))?;

    Ok(Some(size))
}

/// Safely deletes a cargo-nix cache directory with verification
pub fn delete_cargo_nix_dir(cargo_nix_path: &Path, dry_run: bool) -> Result<Option<u64>> {
    // Verify it's actually a cargo-nix directory
    if !is_cargo_nix_dir(cargo_nix_path) {
        return Ok(None);
    }

    if dry_run {
        return Ok(Some(0)); // In dry-run, don't calculate size
    }

    // Calculate size before deletion
    let size = calculate_dir_size(cargo_nix_path).unwrap_or(0);

    // Delete the directory
    fs::remove_dir_all(cargo_nix_path)
        .with_context(|| format!("Failed to delete .cargo-nix directory: {}", cargo_nix_path.display()))?;

    Ok(Some(size))
}

/// Validates a Cargo project by running `cargo metadata --no-deps`
fn validate_project(project_dir: &Path) -> Result<(), String> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to execute cargo metadata: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        // Extract just the first line of the error for cleaner output
        let first_line = error_msg.lines().next().unwrap_or("Invalid project");
        Err(first_line.to_string())
    }
}

/// Cleans a Cargo project and optionally deletes its target directory
pub fn clean_project(
    project_dir: &Path,
    dry_run: bool,
    verbose: bool,
    force: bool,
    strict: bool,
) -> Result<CleanResult> {
    let project_path = project_dir.display().to_string();
    let target_path = project_dir.join("target");

    // Validate the project first unless --force is specified
    if !force {
        if let Err(reason) = validate_project(project_dir) {
            // If validation fails but we're not in strict mode, try to clean target directory anyway
            if !strict && target_path.exists() && is_rust_target_dir(&target_path) {
                if verbose {
                    println!("{} {} - {}", "⊙".yellow(), project_path, "cleaning target only (invalid project config)");
                }

                if dry_run {
                    return Ok(CleanResult {
                        project_path,
                        status: CleanStatus::TargetOnly {
                            space_freed: 0,
                            reason: reason.clone(),
                        },
                    });
                }

                // Calculate and delete target
                let space_freed = calculate_dir_size(&target_path).unwrap_or(0);
                delete_target_dir(&target_path, false)?;

                println!("{} {} (target only)", "⊙".cyan(), project_path);
                return Ok(CleanResult {
                    project_path,
                    status: CleanStatus::TargetOnly { space_freed, reason },
                });
            }

            // In strict mode or no valid target, skip the project
            if verbose {
                println!("{} {} - {}", "⊘".yellow(), project_path, reason);
            }
            return Ok(CleanResult {
                project_path,
                status: CleanStatus::Skipped(reason),
            });
        }
    }

    if dry_run {
        println!("{} {}", "[DRY RUN]".yellow(), project_path);
        return Ok(CleanResult {
            project_path,
            status: CleanStatus::Success { space_freed: None },
        });
    }

    if verbose {
        println!("{} {}", "Cleaning".cyan(), project_path);
    }

    // Calculate total space freed from all target variants
    let mut total_space_freed = 0u64;
    let mut found_any_target = false;

    // List of target directory variants to clean
    let target_variants = ["target", "target-ra"];

    for variant in &target_variants {
        let target_path = project_dir.join(variant);
        if target_path.exists() && is_rust_target_dir(&target_path) {
            found_any_target = true;
            if let Ok(size) = calculate_dir_size(&target_path) {
                total_space_freed += size;
            }
            delete_target_dir(&target_path, dry_run).ok();
        }
    }

    let space_freed = if found_any_target {
        Some(total_space_freed)
    } else {
        None
    };

    println!("{} {}", "✓".green(), project_path);
    Ok(CleanResult {
        project_path,
        status: CleanStatus::Success { space_freed },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_rust_target_dir() {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("target");
        fs::create_dir(&target_dir).unwrap();

        // Should return false without markers
        assert!(!is_rust_target_dir(&target_dir));

        // Should return true with CACHEDIR.TAG
        fs::write(target_dir.join("CACHEDIR.TAG"), "test").unwrap();
        assert!(is_rust_target_dir(&target_dir));

        // Should return false if contains Cargo.toml
        fs::write(target_dir.join("Cargo.toml"), "[package]").unwrap();
        assert!(!is_rust_target_dir(&target_dir));
    }
}
