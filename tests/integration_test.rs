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
    assert_eq!(discovered.projects.len(), 3, "Expected 3 Rust projects");
    assert_eq!(discovered.orphaned_targets.len(), 1, "Expected 1 orphaned target");
    // Note: npm can create nested node_modules (e.g., send/node_modules), so we check >= 2
    assert!(discovered.node_modules.len() >= 2, "Expected at least 2 node_modules directories, found {}", discovered.node_modules.len());
    assert_eq!(discovered.python_venvs.len(), 2, "Expected 2 Python venvs");

    println!("✓ Walker correctly discovered all artifacts:");
    println!("  - {} Rust projects", discovered.projects.len());
    println!("  - {} orphaned targets", discovered.orphaned_targets.len());
    println!("  - {} node_modules", discovered.node_modules.len());
    println!("  - {} Python venvs", discovered.python_venvs.len());
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

    // Verify target directories exist before cleaning
    for project in &discovered.projects {
        let target = project.join("target");
        assert!(dir_exists(&target), "Target directory should exist before cleaning: {:?}", target);
    }

    // Clean each project
    let mut cleaned_count = 0;
    for project in &discovered.projects {
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
    for project in &discovered.projects {
        let target = project.join("target");
        assert!(!dir_exists(&target), "Target directory should be removed after cleaning: {:?}", target);
    }

    assert_eq!(cleaned_count, 3, "Expected 3 projects to be cleaned successfully");

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

    // Verify all artifacts were cleaned
    assert_eq!(rust_cleaned, 3, "Expected 3 Rust projects cleaned");
    assert_eq!(orphaned_cleaned, 1, "Expected 1 orphaned target cleaned");
    assert!(node_cleaned >= 2, "Expected at least 2 node_modules cleaned (found {}, nested may not validate)", node_cleaned);
    assert_eq!(venv_cleaned, 2, "Expected 2 Python venvs cleaned");

    let total_cleaned = rust_cleaned + orphaned_cleaned + node_cleaned + venv_cleaned;
    // Note: Nested node_modules may not pass validation (no parent package.json),
    // so we check that we cleaned at least the main artifacts
    assert!(total_cleaned >= 8, "Expected at least 8 artifacts cleaned, got {}", total_cleaned);

    println!("✓ Full cleanup successful:");
    println!("  - {} Rust projects", rust_cleaned);
    println!("  - {} orphaned targets", orphaned_cleaned);
    println!("  - {} node_modules", node_cleaned);
    println!("  - {} Python venvs", venv_cleaned);
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

    println!("✓ Validation correctly rejects false positives");
}
