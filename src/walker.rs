use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crate::cleaner::{is_node_modules_dir, is_python_venv_dir};

pub struct DiscoveredPaths {
    pub projects: Vec<PathBuf>,
    pub orphaned_targets: Vec<PathBuf>,
    pub node_modules: Vec<PathBuf>,
    pub python_venvs: Vec<PathBuf>,
}

/// Finds all directories containing a Cargo.toml file by walking the given directory
pub fn find_cargo_projects(root: &Path) -> Result<Vec<PathBuf>> {
    let discovered = find_all_rust_artifacts(root)?;
    Ok(discovered.projects)
}

/// Finds both Cargo projects and orphaned target directories
pub fn find_all_rust_artifacts(root: &Path) -> Result<DiscoveredPaths> {
    // Thread-safe collections for results
    let projects = Arc::new(Mutex::new(Vec::new()));
    let orphaned_targets = Arc::new(Mutex::new(Vec::new()));
    let node_modules = Arc::new(Mutex::new(Vec::new()));
    let python_venvs = Arc::new(Mutex::new(Vec::new()));

    // Build the parallel walker
    // Use ignore crate ONLY for parallel walking performance (like ripgrep)
    // Disable ALL gitignore filtering - we rely on validation functions instead:
    // - is_rust_target_dir() checks for CACHEDIR.TAG/.rustc_info.json
    // - is_node_modules_dir() checks for package.json + structure
    // - is_python_venv_dir() checks for pyvenv.cfg + activation + lib
    let walker = WalkBuilder::new(root)
        .follow_links(false)
        .git_ignore(false)        // Don't filter based on .gitignore
        .git_global(false)         // Don't filter based on global gitignore
        .git_exclude(false)        // Don't filter based on .git/info/exclude
        .ignore(false)             // Don't filter based on .ignore files
        .parents(false)            // Don't look at parent directories for ignore files
        .hidden(false)             // Don't filter hidden files/directories (needed for .venv)
        .build_parallel();

    // Walk directories in parallel
    let projects_clone = Arc::clone(&projects);
    let orphaned_clone = Arc::clone(&orphaned_targets);
    let node_modules_clone = Arc::clone(&node_modules);
    let python_venvs_clone = Arc::clone(&python_venvs);

    walker.run(move || {
        let projects = Arc::clone(&projects_clone);
        let orphaned_targets = Arc::clone(&orphaned_clone);
        let node_modules = Arc::clone(&node_modules_clone);
        let python_venvs = Arc::clone(&python_venvs_clone);

        Box::new(move |result| {
            use ignore::WalkState;

            if let Ok(entry) = result {
                let path = entry.path();

                // Check if this is a Cargo.toml file
                if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
                    // Get the parent directory (the project root)
                    if let Some(project_dir) = path.parent() {
                        if let Ok(mut projects) = projects.lock() {
                            projects.push(project_dir.to_path_buf());
                        }
                    }
                }
                // Check if this is a directory
                else if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str());

                    // Check if this is a potentially orphaned target directory
                    // Support "target" and "target-ra" (rust-analyzer cache)
                    let is_target_dir = dir_name.map_or(false, |name| {
                        name == "target" || name == "target-ra"
                    });

                    if is_target_dir {
                        // Verify it's a Rust target by checking for markers
                        let has_cachedir_tag = path.join("CACHEDIR.TAG").exists();
                        let has_rustc_info = path.join(".rustc_info.json").exists();

                        if has_cachedir_tag || has_rustc_info {
                            // Check if parent has Cargo.toml - if not, it's orphaned
                            if let Some(parent) = path.parent() {
                                if !parent.join("Cargo.toml").exists() {
                                    if let Ok(mut orphaned) = orphaned_targets.lock() {
                                        orphaned.push(path.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                    // Check if this is a node_modules directory
                    else if dir_name == Some("node_modules") {
                        if is_node_modules_dir(path) {
                            if let Ok(mut nm) = node_modules.lock() {
                                nm.push(path.to_path_buf());
                            }
                        }
                    }
                    // Check if this is a Python venv directory
                    else if let Some(name) = dir_name {
                        let venv_names = ["venv", ".venv", "env", "ENV", "virtualenv", ".virtualenv"];
                        if venv_names.contains(&name) {
                            if is_python_venv_dir(path) {
                                if let Ok(mut venvs) = python_venvs.lock() {
                                    venvs.push(path.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }

            WalkState::Continue
        })
    });

    // Extract the results from the mutexes
    let projects = Arc::try_unwrap(projects)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Arc"))?
        .into_inner()
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Mutex"))?;

    let orphaned_targets = Arc::try_unwrap(orphaned_targets)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Arc"))?
        .into_inner()
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Mutex"))?;

    let node_modules = Arc::try_unwrap(node_modules)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Arc"))?
        .into_inner()
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Mutex"))?;

    let python_venvs = Arc::try_unwrap(python_venvs)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Arc"))?
        .into_inner()
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Mutex"))?;

    Ok(DiscoveredPaths {
        projects,
        orphaned_targets,
        node_modules,
        python_venvs,
    })
}
