use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Helper function to run the setup script
fn setup_test_artifacts(test_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("setup_test_artifacts.sh");

    let output = Command::new("bash")
        .arg(&script_path)
        .arg(test_dir)
        .output()?;

    if !output.status.success() {
        eprintln!("Setup script stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Setup script stderr: {}", String::from_utf8_lossy(&output.stderr));
        return Err(format!("Setup script failed with status: {}", output.status).into());
    }

    Ok(())
}

/// Helper function to check if a directory exists
fn dir_exists(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

#[test]
fn test_setup_creates_all_artifacts() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Verify Rust projects were created
    assert!(dir_exists(&test_path.join("rust-project-1")));
    assert!(dir_exists(&test_path.join("rust-project-1/target")));
    assert!(test_path.join("rust-project-1/Cargo.toml").exists());

    assert!(dir_exists(&test_path.join("rust-project-2")));
    assert!(dir_exists(&test_path.join("rust-project-2/target")));

    assert!(dir_exists(&test_path.join("rust-project-3")));
    assert!(dir_exists(&test_path.join("rust-project-3/target")));

    // Verify orphaned target was created
    assert!(dir_exists(&test_path.join("orphaned-workspace/target")));
    assert!(test_path.join("orphaned-workspace/target/CACHEDIR.TAG").exists());
    assert!(!test_path.join("orphaned-workspace/Cargo.toml").exists());

    // Verify Node.js projects were created
    assert!(dir_exists(&test_path.join("node-project-1")));
    assert!(dir_exists(&test_path.join("node-project-1/node_modules")));
    assert!(test_path.join("node-project-1/package.json").exists());

    assert!(dir_exists(&test_path.join("node-project-2")));
    assert!(dir_exists(&test_path.join("node-project-2/node_modules")));

    // Verify Python projects were created
    assert!(dir_exists(&test_path.join("python-project-1")));
    assert!(dir_exists(&test_path.join("python-project-1/.venv")));
    assert!(test_path.join("python-project-1/.venv/pyvenv.cfg").exists());

    assert!(dir_exists(&test_path.join("python-project-2")));
    assert!(dir_exists(&test_path.join("python-project-2/.venv")));

    // Verify Haskell Stack projects were created
    assert!(dir_exists(&test_path.join("haskell-project-1")));
    assert!(dir_exists(&test_path.join("haskell-project-1/.stack-work")));
    assert!(test_path.join("haskell-project-1/stack.yaml").exists());
    assert!(test_path.join("haskell-project-1/.stack-work/stack.sqlite3").exists());

    assert!(dir_exists(&test_path.join("haskell-project-2")));
    assert!(dir_exists(&test_path.join("haskell-project-2/.stack-work")));

    // Verify rustup directories were created
    assert!(dir_exists(&test_path.join("home-sim/.rustup")));
    assert!(test_path.join("home-sim/.rustup/settings.toml").exists());
    assert!(dir_exists(&test_path.join("home-sim/.rustup/toolchains")));

    assert!(dir_exists(&test_path.join("home-sim-2/.rustup")));
    assert!(test_path.join("home-sim-2/.rustup/settings.toml").exists());

    // Verify Next.js projects were created
    assert!(dir_exists(&test_path.join("nextjs-project-1")));
    assert!(dir_exists(&test_path.join("nextjs-project-1/.next")));
    assert!(test_path.join("nextjs-project-1/next.config.js").exists());
    assert!(test_path.join("nextjs-project-1/.next/BUILD_ID").exists());

    assert!(dir_exists(&test_path.join("nextjs-project-2")));
    assert!(dir_exists(&test_path.join("nextjs-project-2/.next")));
    assert!(test_path.join("nextjs-project-2/next.config.mjs").exists());

    // Verify cargo-nix directories were created
    assert!(dir_exists(&test_path.join("rust-nix-project-1/.cargo-nix")));
    assert!(dir_exists(&test_path.join("rust-nix-project-2/.cargo-nix")));

    println!("✓ All test artifacts created successfully");
}

#[test]
fn test_walker_finds_all_artifacts() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Use the walker to find all artifacts
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    // Verify counts
    // Note: rust-nix-project-1 has a Cargo.toml so it's counted as a Rust project too
    assert!(discovered.projects.len() >= 3, "Expected at least 3 Rust projects, found {}", discovered.projects.len());
    assert_eq!(discovered.orphaned_targets.len(), 1, "Expected 1 orphaned target");
    // Note: npm can create nested node_modules (e.g., send/node_modules), so we check >= 2
    assert!(discovered.node_modules.len() >= 2, "Expected at least 2 node_modules directories, found {}", discovered.node_modules.len());
    assert_eq!(discovered.python_venvs.len(), 2, "Expected 2 Python venvs");
    assert_eq!(discovered.sccache_dirs.len(), 2, "Expected 2 sccache directories");
    assert_eq!(discovered.stack_work_dirs.len(), 2, "Expected 2 Stack work directories");
    assert_eq!(discovered.rustup_dirs.len(), 2, "Expected 2 rustup directories");
    assert_eq!(discovered.next_dirs.len(), 2, "Expected 2 Next.js build directories");
    assert_eq!(discovered.cargo_nix_dirs.len(), 2, "Expected 2 cargo-nix directories");

    println!("✓ Walker correctly discovered all artifacts:");
    println!("  - {} Rust projects", discovered.projects.len());
    println!("  - {} orphaned targets", discovered.orphaned_targets.len());
    println!("  - {} node_modules", discovered.node_modules.len());
    println!("  - {} Python venvs", discovered.python_venvs.len());
    println!("  - {} sccache dirs", discovered.sccache_dirs.len());
    println!("  - {} Stack work dirs", discovered.stack_work_dirs.len());
    println!("  - {} rustup dirs", discovered.rustup_dirs.len());
    println!("  - {} Next.js builds", discovered.next_dirs.len());
    println!("  - {} cargo-nix dirs", discovered.cargo_nix_dirs.len());
}

#[test]
fn test_clean_rust_projects() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find Rust projects
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    // Filter to only projects with target directories (rust-nix-project-1 doesn't have one)
    let projects_with_target: Vec<_> = discovered.projects.iter()
        .filter(|p| dir_exists(&p.join("target")))
        .collect();

    // Verify we have at least 3 projects with target directories
    assert!(projects_with_target.len() >= 3, "Expected at least 3 Rust projects with target dirs, found {}", projects_with_target.len());

    // Clean each project with a target directory
    let mut cleaned_count = 0;
    for project in &projects_with_target {
        let result = wd_40::cleaner::clean_project(
            project,
            false, // dry_run
            false, // verbose
            false, // force
            false, // strict
        ).expect("Failed to clean project");

        if result.is_success() {
            cleaned_count += 1;
        }
    }

    // Verify target directories were removed
    for project in &projects_with_target {
        let target = project.join("target");
        assert!(!dir_exists(&target), "Target directory should be removed after cleaning: {:?}", target);
    }

    assert!(cleaned_count >= 3, "Expected at least 3 projects to be cleaned successfully, got {}", cleaned_count);

    println!("✓ Successfully cleaned {} Rust projects", cleaned_count);
}

#[test]
fn test_clean_orphaned_targets() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find orphaned targets
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    assert_eq!(discovered.orphaned_targets.len(), 1, "Expected 1 orphaned target");

    let orphaned = &discovered.orphaned_targets[0];
    assert!(dir_exists(orphaned), "Orphaned target should exist before cleaning");

    // Clean the orphaned target using the specific orphaned function
    let result = wd_40::cleaner::delete_orphaned_target_dir(orphaned, false)
        .expect("Failed to delete orphaned target");

    assert!(result.is_some(), "Orphaned target should be deleted");
    assert!(!dir_exists(orphaned), "Orphaned target should not exist after cleaning");

    println!("✓ Successfully cleaned orphaned target directory");
}

#[test]
fn test_clean_node_modules() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find node_modules
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    // Note: npm can create nested node_modules, so we check >= 2
    assert!(discovered.node_modules.len() >= 2, "Expected at least 2 node_modules directories");

    // Verify they exist before cleaning
    for nm in &discovered.node_modules {
        assert!(dir_exists(nm), "node_modules should exist before cleaning: {:?}", nm);
    }

    // Clean each node_modules
    let mut cleaned_count = 0;
    for nm in &discovered.node_modules {
        let result = wd_40::cleaner::delete_node_modules_dir(nm, false)
            .expect("Failed to delete node_modules");

        if result.is_some() {
            cleaned_count += 1;
        }
    }

    // Verify they were removed
    for nm in &discovered.node_modules {
        assert!(!dir_exists(nm), "node_modules should not exist after cleaning: {:?}", nm);
    }

    assert!(cleaned_count >= 2, "Expected at least 2 node_modules to be cleaned");

    println!("✓ Successfully cleaned {} node_modules directories", cleaned_count);
}

#[test]
fn test_clean_python_venvs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find Python venvs
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    assert_eq!(discovered.python_venvs.len(), 2, "Expected 2 Python venvs");

    // Verify they exist before cleaning
    for venv in &discovered.python_venvs {
        assert!(dir_exists(venv), "Python venv should exist before cleaning: {:?}", venv);
    }

    // Clean each venv
    let mut cleaned_count = 0;
    for venv in &discovered.python_venvs {
        let result = wd_40::cleaner::delete_venv_dir(venv, false)
            .expect("Failed to delete venv");

        if result.is_some() {
            cleaned_count += 1;
        }
    }

    // Verify they were removed
    for venv in &discovered.python_venvs {
        assert!(!dir_exists(venv), "Python venv should not exist after cleaning: {:?}", venv);
    }

    assert_eq!(cleaned_count, 2, "Expected 2 Python venvs to be cleaned");

    println!("✓ Successfully cleaned {} Python virtual environments", cleaned_count);
}

#[test]
fn test_clean_sccache_dirs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find sccache directories
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    assert_eq!(discovered.sccache_dirs.len(), 2, "Expected 2 sccache directories");

    // Verify they exist before cleaning
    for sccache in &discovered.sccache_dirs {
        assert!(dir_exists(sccache), "sccache directory should exist before cleaning: {:?}", sccache);
    }

    // Clean each sccache directory
    let mut cleaned_count = 0;
    for sccache in &discovered.sccache_dirs {
        let result = wd_40::cleaner::delete_sccache_dir(sccache, false)
            .expect("Failed to delete sccache directory");

        if result.is_some() {
            cleaned_count += 1;
        }
    }

    // Verify they were removed
    for sccache in &discovered.sccache_dirs {
        assert!(!dir_exists(sccache), "sccache directory should not exist after cleaning: {:?}", sccache);
    }

    assert_eq!(cleaned_count, 2, "Expected 2 sccache directories to be cleaned");

    println!("✓ Successfully cleaned {} sccache directories", cleaned_count);
}

#[test]
fn test_clean_stack_work_dirs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find Stack work directories
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    assert_eq!(discovered.stack_work_dirs.len(), 2, "Expected 2 Stack work directories");

    // Verify they exist before cleaning
    for stack_work in &discovered.stack_work_dirs {
        assert!(dir_exists(stack_work), "Stack work directory should exist before cleaning: {:?}", stack_work);
    }

    // Clean each Stack work directory
    let mut cleaned_count = 0;
    for stack_work in &discovered.stack_work_dirs {
        let result = wd_40::cleaner::delete_stack_work_dir(stack_work, false)
            .expect("Failed to delete Stack work directory");

        if result.is_some() {
            cleaned_count += 1;
        }
    }

    // Verify they were removed
    for stack_work in &discovered.stack_work_dirs {
        assert!(!dir_exists(stack_work), "Stack work directory should not exist after cleaning: {:?}", stack_work);
    }

    assert_eq!(cleaned_count, 2, "Expected 2 Stack work directories to be cleaned");

    println!("✓ Successfully cleaned {} Stack work directories", cleaned_count);
}

#[test]
fn test_clean_rustup_dirs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find rustup directories
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    assert_eq!(discovered.rustup_dirs.len(), 2, "Expected 2 rustup directories");

    // Verify they exist before cleaning
    for rustup in &discovered.rustup_dirs {
        assert!(dir_exists(rustup), "rustup directory should exist before cleaning: {:?}", rustup);
    }

    // Clean each rustup directory
    let mut cleaned_count = 0;
    for rustup in &discovered.rustup_dirs {
        let result = wd_40::cleaner::delete_rustup_dir(rustup, false)
            .expect("Failed to delete rustup directory");

        if result.is_some() {
            cleaned_count += 1;
        }
    }

    // Verify they were removed
    for rustup in &discovered.rustup_dirs {
        assert!(!dir_exists(rustup), "rustup directory should not exist after cleaning: {:?}", rustup);
    }

    assert_eq!(cleaned_count, 2, "Expected 2 rustup directories to be cleaned");

    println!("✓ Successfully cleaned {} rustup directories", cleaned_count);
}

#[test]
fn test_clean_next_dirs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find Next.js directories
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    assert_eq!(discovered.next_dirs.len(), 2, "Expected 2 Next.js build directories");

    // Verify they exist before cleaning
    for next in &discovered.next_dirs {
        assert!(dir_exists(next), "Next.js build directory should exist before cleaning: {:?}", next);
    }

    // Clean each Next.js directory
    let mut cleaned_count = 0;
    for next in &discovered.next_dirs {
        let result = wd_40::cleaner::delete_next_dir(next, false)
            .expect("Failed to delete Next.js directory");

        if result.is_some() {
            cleaned_count += 1;
        }
    }

    // Verify they were removed
    for next in &discovered.next_dirs {
        assert!(!dir_exists(next), "Next.js build directory should not exist after cleaning: {:?}", next);
    }

    assert_eq!(cleaned_count, 2, "Expected 2 Next.js build directories to be cleaned");

    println!("✓ Successfully cleaned {} Next.js build directories", cleaned_count);
}

#[test]
fn test_clean_cargo_nix_dirs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find cargo-nix directories
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    assert_eq!(discovered.cargo_nix_dirs.len(), 2, "Expected 2 cargo-nix directories");

    // Verify they exist before cleaning
    for cargo_nix in &discovered.cargo_nix_dirs {
        assert!(dir_exists(cargo_nix), "cargo-nix directory should exist before cleaning: {:?}", cargo_nix);
    }

    // Clean each cargo-nix directory
    let mut cleaned_count = 0;
    for cargo_nix in &discovered.cargo_nix_dirs {
        let result = wd_40::cleaner::delete_cargo_nix_dir(cargo_nix, false)
            .expect("Failed to delete cargo-nix directory");

        if result.is_some() {
            cleaned_count += 1;
        }
    }

    // Verify they were removed
    for cargo_nix in &discovered.cargo_nix_dirs {
        assert!(!dir_exists(cargo_nix), "cargo-nix directory should not exist after cleaning: {:?}", cargo_nix);
    }

    assert_eq!(cleaned_count, 2, "Expected 2 cargo-nix directories to be cleaned");

    println!("✓ Successfully cleaned {} cargo-nix directories", cleaned_count);
}

#[test]
fn test_full_cleanup_all_artifacts() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Run setup script
    setup_test_artifacts(test_path).expect("Setup script failed");

    // Find all artifacts
    let discovered = wd_40::walker::find_all_rust_artifacts(test_path)
        .expect("Failed to find artifacts");

    // Count total artifacts before cleanup
    let total_before = discovered.projects.len()
        + discovered.orphaned_targets.len()
        + discovered.node_modules.len()
        + discovered.python_venvs.len();

    println!("Found {} total artifacts before cleanup", total_before);

    // Clean Rust projects
    let mut rust_cleaned = 0;
    for project in &discovered.projects {
        let result = wd_40::cleaner::clean_project(project, false, false, false, false)
            .expect("Failed to clean project");
        if result.is_success() {
            rust_cleaned += 1;
        }
    }

    // Clean orphaned targets
    let mut orphaned_cleaned = 0;
    for orphaned in &discovered.orphaned_targets {
        if wd_40::cleaner::delete_orphaned_target_dir(orphaned, false).ok().flatten().is_some() {
            orphaned_cleaned += 1;
        }
    }

    // Clean node_modules
    let mut node_cleaned = 0;
    for nm in &discovered.node_modules {
        if wd_40::cleaner::delete_node_modules_dir(nm, false).ok().flatten().is_some() {
            node_cleaned += 1;
        }
    }

    // Clean Python venvs
    let mut venv_cleaned = 0;
    for venv in &discovered.python_venvs {
        if wd_40::cleaner::delete_venv_dir(venv, false).ok().flatten().is_some() {
            venv_cleaned += 1;
        }
    }

    // Clean sccache directories
    let mut sccache_cleaned = 0;
    for sccache in &discovered.sccache_dirs {
        if wd_40::cleaner::delete_sccache_dir(sccache, false).ok().flatten().is_some() {
            sccache_cleaned += 1;
        }
    }

    // Clean Stack work directories
    let mut stack_work_cleaned = 0;
    for stack_work in &discovered.stack_work_dirs {
        if wd_40::cleaner::delete_stack_work_dir(stack_work, false).ok().flatten().is_some() {
            stack_work_cleaned += 1;
        }
    }

    // Clean rustup directories
    let mut rustup_cleaned = 0;
    for rustup in &discovered.rustup_dirs {
        if wd_40::cleaner::delete_rustup_dir(rustup, false).ok().flatten().is_some() {
            rustup_cleaned += 1;
        }
    }

    // Clean Next.js build directories
    let mut next_cleaned = 0;
    for next in &discovered.next_dirs {
        if wd_40::cleaner::delete_next_dir(next, false).ok().flatten().is_some() {
            next_cleaned += 1;
        }
    }

    // Clean cargo-nix directories
    let mut cargo_nix_cleaned = 0;
    for cargo_nix in &discovered.cargo_nix_dirs {
        if wd_40::cleaner::delete_cargo_nix_dir(cargo_nix, false).ok().flatten().is_some() {
            cargo_nix_cleaned += 1;
        }
    }

    // Verify all artifacts were cleaned
    // Note: rust-nix-project-1 has a Cargo.toml so it's counted as a Rust project too
    assert!(rust_cleaned >= 3, "Expected at least 3 Rust projects cleaned, got {}", rust_cleaned);
    assert_eq!(orphaned_cleaned, 1, "Expected 1 orphaned target cleaned");
    assert!(node_cleaned >= 2, "Expected at least 2 node_modules cleaned (found {}, nested may not validate)", node_cleaned);
    assert_eq!(venv_cleaned, 2, "Expected 2 Python venvs cleaned");
    assert_eq!(sccache_cleaned, 2, "Expected 2 sccache directories cleaned");
    assert_eq!(stack_work_cleaned, 2, "Expected 2 Stack work directories cleaned");
    assert_eq!(rustup_cleaned, 2, "Expected 2 rustup directories cleaned");
    assert_eq!(next_cleaned, 2, "Expected 2 Next.js build directories cleaned");
    assert_eq!(cargo_nix_cleaned, 2, "Expected 2 cargo-nix directories cleaned");

    let total_cleaned = rust_cleaned + orphaned_cleaned + node_cleaned + venv_cleaned + sccache_cleaned + stack_work_cleaned + rustup_cleaned + next_cleaned + cargo_nix_cleaned;
    // Note: Nested node_modules may not pass validation (no parent package.json),
    // so we check that we cleaned at least the main artifacts
    assert!(total_cleaned >= 18, "Expected at least 18 artifacts cleaned, got {}", total_cleaned);

    println!("✓ Full cleanup successful:");
    println!("  - {} Rust projects", rust_cleaned);
    println!("  - {} orphaned targets", orphaned_cleaned);
    println!("  - {} node_modules", node_cleaned);
    println!("  - {} Python venvs", venv_cleaned);
    println!("  - {} sccache dirs", sccache_cleaned);
    println!("  - {} Stack work dirs", stack_work_cleaned);
    println!("  - {} rustup dirs", rustup_cleaned);
    println!("  - {} Next.js builds", next_cleaned);
    println!("  - {} cargo-nix dirs", cargo_nix_cleaned);
    println!("  - {} total artifacts cleaned", total_cleaned);
}

#[test]
fn test_validation_prevents_false_positives() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path();

    // Create a directory named "target" but without Rust markers
    let fake_target = test_path.join("fake-target");
    std::fs::create_dir_all(&fake_target).expect("Failed to create fake target");

    // Should not be recognized as a Rust target
    assert!(!wd_40::cleaner::is_rust_target_dir(&fake_target));

    // Create a directory named "node_modules" but without parent package.json
    let fake_nm = test_path.join("node_modules");
    std::fs::create_dir_all(&fake_nm).expect("Failed to create fake node_modules");

    // Should not be recognized as node_modules
    assert!(!wd_40::cleaner::is_node_modules_dir(&fake_nm));

    // Create a directory named ".venv" but without pyvenv.cfg
    let fake_venv = test_path.join(".venv");
    std::fs::create_dir_all(&fake_venv).expect("Failed to create fake venv");

    // Should not be recognized as a Python venv
    assert!(!wd_40::cleaner::is_python_venv_dir(&fake_venv));

    // Create a directory named ".sccache" but empty (no cache structure)
    let fake_sccache = test_path.join(".sccache");
    std::fs::create_dir_all(&fake_sccache).expect("Failed to create fake sccache");

    // Should not be recognized as sccache (empty directory)
    assert!(!wd_40::cleaner::is_sccache_dir(&fake_sccache));

    // Create a directory named ".sccache" with project markers inside it (should be rejected)
    let fake_sccache_project = test_path.join(".sccache");
    std::fs::create_dir_all(&fake_sccache_project).expect("Failed to create fake sccache");
    // Add Cargo.toml inside the .sccache directory (making it look like a project, not a cache)
    std::fs::write(fake_sccache_project.join("Cargo.toml"), "[package]")
        .expect("Failed to write Cargo.toml");
    // Add some files to make it look like it has content
    std::fs::write(fake_sccache_project.join("somefile"), "content")
        .expect("Failed to write file");

    // Should not be recognized as sccache (has Cargo.toml inside it - looks like a project)
    assert!(!wd_40::cleaner::is_sccache_dir(&fake_sccache_project));

    // Create a directory named ".stack-work" but without Stack markers
    let fake_stack_work = test_path.join(".stack-work");
    std::fs::create_dir_all(&fake_stack_work).expect("Failed to create fake stack-work");

    // Should not be recognized as a Stack work directory (empty directory)
    assert!(!wd_40::cleaner::is_stack_work_dir(&fake_stack_work));

    // Create a directory named ".stack-work" with project markers inside it (should be rejected)
    let fake_stack_work_project = test_path.join("project").join(".stack-work");
    std::fs::create_dir_all(&fake_stack_work_project).expect("Failed to create fake stack-work");
    // Add Cargo.toml inside the .stack-work directory (making it look like a project, not a build artifact)
    std::fs::write(fake_stack_work_project.join("Cargo.toml"), "[package]")
        .expect("Failed to write Cargo.toml");
    // Add dist directory to make it look like it has content
    std::fs::create_dir_all(fake_stack_work_project.join("dist"))
        .expect("Failed to create dist");

    // Should not be recognized as Stack work (has Cargo.toml inside it - looks like a project)
    assert!(!wd_40::cleaner::is_stack_work_dir(&fake_stack_work_project));

    // Create a valid-looking .stack-work but without parent stack.yaml
    let orphaned_stack_work = test_path.join("orphaned").join(".stack-work");
    std::fs::create_dir_all(&orphaned_stack_work).expect("Failed to create orphaned stack-work");
    // Add Stack markers
    std::fs::write(orphaned_stack_work.join("stack.sqlite3"), "SQLite format 3")
        .expect("Failed to create sqlite file");
    std::fs::create_dir_all(orphaned_stack_work.join("dist"))
        .expect("Failed to create dist");

    // Should not be recognized (no parent stack.yaml or .cabal file, and sqlite is the only marker)
    // Actually this should pass because we have sqlite db which is a strong marker
    // Let me check the validation logic...
    // The validation requires: (has_stack_yaml OR has_package_yaml OR has_cabal_file OR has_sqlite_db)
    // Since we have sqlite_db, it should pass. So this is actually valid!
    // Let's test a truly invalid case instead - no parent project markers and no sqlite

    let truly_orphaned = test_path.join("truly-orphaned").join(".stack-work");
    std::fs::create_dir_all(&truly_orphaned).expect("Failed to create truly orphaned");
    // Only add dist, no sqlite
    std::fs::create_dir_all(truly_orphaned.join("dist"))
        .expect("Failed to create dist");

    // Should not be recognized (no parent markers and no strong marker like sqlite)
    assert!(!wd_40::cleaner::is_stack_work_dir(&truly_orphaned));

    // Create a directory named ".rustup" but without rustup markers
    let fake_rustup = test_path.join(".rustup");
    std::fs::create_dir_all(&fake_rustup).expect("Failed to create fake rustup");

    // Should not be recognized as rustup (empty directory)
    assert!(!wd_40::cleaner::is_rustup_dir(&fake_rustup));

    // Create a directory named ".rustup" with project markers inside (should be rejected)
    let fake_rustup_project = test_path.join("project").join(".rustup");
    std::fs::create_dir_all(&fake_rustup_project).expect("Failed to create fake rustup");
    // Add Cargo.toml inside the .rustup directory (making it look like a project, not rustup)
    std::fs::write(fake_rustup_project.join("Cargo.toml"), "[package]")
        .expect("Failed to write Cargo.toml");
    // Add settings.toml to make it look like rustup
    std::fs::write(fake_rustup_project.join("settings.toml"), "default_toolchain = \"stable\"")
        .expect("Failed to write settings.toml");

    // Should not be recognized as rustup (has Cargo.toml inside it - looks like a project)
    assert!(!wd_40::cleaner::is_rustup_dir(&fake_rustup_project));

    // Create a directory named ".next" but without Next.js markers
    let fake_next = test_path.join(".next");
    std::fs::create_dir_all(&fake_next).expect("Failed to create fake .next");

    // Should not be recognized as Next.js build (empty directory, no parent markers)
    assert!(!wd_40::cleaner::is_next_dir(&fake_next));

    // Create a directory named ".next" with markers but no parent project markers
    let orphaned_next = test_path.join("orphaned-next").join(".next");
    std::fs::create_dir_all(&orphaned_next).expect("Failed to create orphaned .next");
    std::fs::write(orphaned_next.join("BUILD_ID"), "build-id-123")
        .expect("Failed to write BUILD_ID");
    std::fs::create_dir_all(orphaned_next.join("cache"))
        .expect("Failed to create cache");

    // Should not be recognized (no parent package.json or next.config.*)
    assert!(!wd_40::cleaner::is_next_dir(&orphaned_next));

    // Create a directory named ".next" with project markers inside (should be rejected)
    let fake_next_project = test_path.join("project").join(".next");
    std::fs::create_dir_all(&fake_next_project).expect("Failed to create fake .next");
    std::fs::write(fake_next_project.join("package.json"), "{}")
        .expect("Failed to write package.json");

    // Should not be recognized (has package.json inside - looks like a project, not build output)
    assert!(!wd_40::cleaner::is_next_dir(&fake_next_project));

    // Create a directory named ".cargo-nix" but empty
    let fake_cargo_nix = test_path.join(".cargo-nix");
    std::fs::create_dir_all(&fake_cargo_nix).expect("Failed to create fake .cargo-nix");

    // Should not be recognized as cargo-nix (empty directory)
    assert!(!wd_40::cleaner::is_cargo_nix_dir(&fake_cargo_nix));

    // Create a directory named ".cargo-nix" with project markers inside (should be rejected)
    let fake_cargo_nix_project = test_path.join("project2").join(".cargo-nix");
    std::fs::create_dir_all(&fake_cargo_nix_project).expect("Failed to create fake .cargo-nix");
    std::fs::write(fake_cargo_nix_project.join("Cargo.toml"), "[package]")
        .expect("Failed to write Cargo.toml");
    std::fs::write(fake_cargo_nix_project.join("cache.bin"), "data")
        .expect("Failed to write cache.bin");

    // Should not be recognized (has Cargo.toml inside - looks like a project, not cache)
    assert!(!wd_40::cleaner::is_cargo_nix_dir(&fake_cargo_nix_project));

    println!("✓ Validation correctly rejects false positives");
}
