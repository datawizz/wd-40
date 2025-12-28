pub mod cleaner;
mod logging;
pub mod walker;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use logging::Logger;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "wd-40",
    about = "A CLI tool to recursively find and clean Rust, Node.js, and Python project artifacts",
    version
)]
struct Cli {
    /// Directory to search for project artifacts (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Show what would be cleaned without actually executing
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Show detailed output
    #[arg(short, long)]
    verbose: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    no_confirm: bool,

    /// Force cleaning even if project validation fails
    #[arg(short, long)]
    force: bool,

    /// Strict mode: skip projects with invalid configurations (don't attempt target-only cleaning)
    #[arg(short, long)]
    strict: bool,

    /// Clean only orphaned target directories (no parent Cargo.toml)
    #[arg(long)]
    orphaned_only: bool,

    /// Clean only Rust projects and artifacts
    #[arg(long)]
    rust_only: bool,

    /// Clean only Node.js node_modules directories
    #[arg(long)]
    node_only: bool,

    /// Clean only Python virtual environments
    #[arg(long)]
    python_only: bool,

    /// Clean only Haskell Stack projects
    #[arg(long)]
    haskell_only: bool,

    /// Clean only Rust toolchain (.rustup) directories
    #[arg(long)]
    rustup_only: bool,

    /// Clean only Next.js build (.next) directories
    #[arg(long)]
    next_only: bool,

    /// Clean only cargo-nix (.cargo-nix) directories
    #[arg(long)]
    cargo_nix_only: bool,

    /// Custom log file path (default: ~/.cache/wd-40/clean-<timestamp>.log)
    #[arg(long)]
    log_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // Initialize logger
    let mut logger = Logger::new(args.log_file)?;

    println!("{}", "🛢️  WD-40 - Project Artifact Cleaner".bold().cyan());
    println!();

    // Canonicalize the path
    let root_path = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());

    if args.verbose {
        println!(
            "{} {}",
            "Searching for project artifacts in:".cyan(),
            root_path.display()
        );
    }

    // Find all artifacts (Rust, Node.js, Python)
    let discovered = walker::find_all_rust_artifacts(&root_path)?;

    // Decide what to process based on flags
    let (projects_to_clean, orphaned_to_clean, node_modules_to_clean, venvs_to_clean, sccache_to_clean, stack_work_to_clean, rustup_to_clean, next_to_clean, cargo_nix_to_clean) =
        if args.orphaned_only {
            // Only clean orphaned Rust targets
            (Vec::new(), discovered.orphaned_targets, Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())
        } else if args.rust_only {
            // Only clean Rust artifacts
            (discovered.projects, discovered.orphaned_targets, Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())
        } else if args.node_only {
            // Only clean Node.js artifacts
            (Vec::new(), Vec::new(), discovered.node_modules, Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())
        } else if args.python_only {
            // Only clean Python artifacts
            (Vec::new(), Vec::new(), Vec::new(), discovered.python_venvs, Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new())
        } else if args.haskell_only {
            // Only clean Haskell Stack artifacts
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), discovered.stack_work_dirs, Vec::new(), Vec::new(), Vec::new())
        } else if args.rustup_only {
            // Only clean rustup directories
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), discovered.rustup_dirs, Vec::new(), Vec::new())
        } else if args.next_only {
            // Only clean Next.js build directories
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), discovered.next_dirs, Vec::new())
        } else if args.cargo_nix_only {
            // Only clean cargo-nix directories
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), discovered.cargo_nix_dirs)
        } else {
            // Clean everything by default
            (discovered.projects, discovered.orphaned_targets, discovered.node_modules, discovered.python_venvs, discovered.sccache_dirs, discovered.stack_work_dirs, discovered.rustup_dirs, discovered.next_dirs, discovered.cargo_nix_dirs)
        };

    if projects_to_clean.is_empty()
        && orphaned_to_clean.is_empty()
        && node_modules_to_clean.is_empty()
        && venvs_to_clean.is_empty()
        && sccache_to_clean.is_empty()
        && stack_work_to_clean.is_empty()
        && rustup_to_clean.is_empty()
        && next_to_clean.is_empty()
        && cargo_nix_to_clean.is_empty()
    {
        println!("{}", "No artifacts found.".yellow());
        logger.log_found_projects(0, &[])?;
        return Ok(());
    }

    // Show what was found
    if !projects_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".green(),
            projects_to_clean.len(),
            if projects_to_clean.len() == 1 {
                "Rust project"
            } else {
                "Rust projects"
            }
        );
        if args.verbose {
            for project in &projects_to_clean {
                println!("  {}", project.display());
            }
        }
    }

    if !orphaned_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".yellow(),
            orphaned_to_clean.len(),
            if orphaned_to_clean.len() == 1 {
                "orphaned target directory"
            } else {
                "orphaned target directories"
            }
        );
        if args.verbose || args.orphaned_only {
            for orphaned in &orphaned_to_clean {
                println!("  {}", orphaned.display());
            }
        }
    }

    if !node_modules_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".green(),
            node_modules_to_clean.len(),
            if node_modules_to_clean.len() == 1 {
                "node_modules directory"
            } else {
                "node_modules directories"
            }
        );
        if args.verbose {
            for nm in &node_modules_to_clean {
                println!("  {}", nm.display());
            }
        }
    }

    if !venvs_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".green(),
            venvs_to_clean.len(),
            if venvs_to_clean.len() == 1 {
                "Python virtual environment"
            } else {
                "Python virtual environments"
            }
        );
        if args.verbose {
            for venv in &venvs_to_clean {
                println!("  {}", venv.display());
            }
        }
    }

    if !sccache_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".green(),
            sccache_to_clean.len(),
            if sccache_to_clean.len() == 1 {
                "sccache directory"
            } else {
                "sccache directories"
            }
        );
        if args.verbose {
            for sccache in &sccache_to_clean {
                println!("  {}", sccache.display());
            }
        }
    }

    if !stack_work_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".green(),
            stack_work_to_clean.len(),
            if stack_work_to_clean.len() == 1 {
                "Stack work directory"
            } else {
                "Stack work directories"
            }
        );
        if args.verbose {
            for stack_work in &stack_work_to_clean {
                println!("  {}", stack_work.display());
            }
        }
    }

    if !rustup_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".green(),
            rustup_to_clean.len(),
            if rustup_to_clean.len() == 1 {
                "rustup directory"
            } else {
                "rustup directories"
            }
        );
        if args.verbose {
            for rustup in &rustup_to_clean {
                println!("  {}", rustup.display());
            }
        }
    }

    if !next_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".green(),
            next_to_clean.len(),
            if next_to_clean.len() == 1 {
                "Next.js build directory"
            } else {
                "Next.js build directories"
            }
        );
        if args.verbose {
            for next in &next_to_clean {
                println!("  {}", next.display());
            }
        }
    }

    if !cargo_nix_to_clean.is_empty() {
        println!(
            "{} {} {}",
            "Found".green(),
            cargo_nix_to_clean.len(),
            if cargo_nix_to_clean.len() == 1 {
                "cargo-nix directory"
            } else {
                "cargo-nix directories"
            }
        );
        if args.verbose {
            for cargo_nix in &cargo_nix_to_clean {
                println!("  {}", cargo_nix.display());
            }
        }
    }

    // Log found artifacts
    logger.log_found_projects(projects_to_clean.len(), &projects_to_clean)?;
    if !orphaned_to_clean.is_empty() {
        logger.log_found_orphaned(orphaned_to_clean.len(), &orphaned_to_clean)?;
    }
    if !node_modules_to_clean.is_empty() {
        logger.log_found_node_modules(node_modules_to_clean.len(), &node_modules_to_clean)?;
    }
    if !venvs_to_clean.is_empty() {
        logger.log_found_venvs(venvs_to_clean.len(), &venvs_to_clean)?;
    }
    if !sccache_to_clean.is_empty() {
        logger.log_found_sccache(sccache_to_clean.len(), &sccache_to_clean)?;
    }
    if !stack_work_to_clean.is_empty() {
        logger.log_found_stack_work(stack_work_to_clean.len(), &stack_work_to_clean)?;
    }
    if !rustup_to_clean.is_empty() {
        logger.log_found_rustup(rustup_to_clean.len(), &rustup_to_clean)?;
    }
    if !next_to_clean.is_empty() {
        logger.log_found_next(next_to_clean.len(), &next_to_clean)?;
    }
    if !cargo_nix_to_clean.is_empty() {
        logger.log_found_cargo_nix(cargo_nix_to_clean.len(), &cargo_nix_to_clean)?;
    }

    // Ask for confirmation unless --no-confirm is set
    if !args.no_confirm && !args.dry_run {
        println!("\n{}", "Proceed with cleaning? (y/N)".yellow());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted.".red());
            return Ok(());
        }
    }

    println!(); // Empty line for better readability

    logger.log_cleaning_start()?;

    // Clean each project
    let mut results = Vec::new();
    let mut total_space_freed = 0u64;
    let mut orphaned_cleaned = 0usize;
    let mut node_modules_cleaned = 0usize;
    let mut venvs_cleaned = 0usize;
    let mut sccache_cleaned = 0usize;
    let mut stack_work_cleaned = 0usize;
    let mut rustup_cleaned = 0usize;
    let mut next_cleaned = 0usize;
    let mut cargo_nix_cleaned = 0usize;

    for project in &projects_to_clean {
        let result = cleaner::clean_project(project, args.dry_run, args.verbose, args.force, args.strict)?;

        // Log the result
        match &result.status {
            cleaner::CleanStatus::Success { space_freed } => {
                logger.log_success(&result.project_path, *space_freed)?;
                if let Some(bytes) = space_freed {
                    total_space_freed += bytes;
                }
            }
            cleaner::CleanStatus::TargetOnly { space_freed, reason } => {
                logger.log_target_only(&result.project_path, *space_freed, reason)?;
                total_space_freed += space_freed;
            }
            cleaner::CleanStatus::Skipped(reason) => {
                logger.log_skipped(&result.project_path, reason)?;
            }
            cleaner::CleanStatus::Failed(error) => {
                logger.log_failed(&result.project_path, error)?;
            }
        }

        results.push(result);
    }

    // Clean orphaned target directories
    for orphaned in &orphaned_to_clean {
        if args.dry_run {
            println!("{} {}", "[DRY RUN ORPHANED]".yellow(), orphaned.display());
        } else {
            let space_freed = cleaner::calculate_dir_size(orphaned).unwrap_or(0);
            match cleaner::delete_orphaned_target_dir(orphaned, args.dry_run) {
                Ok(Some(_)) => {
                    println!("{} {} (orphaned)", "⊗".cyan(), orphaned.display());
                    logger.log_orphaned_cleaned(&orphaned.display().to_string(), space_freed)?;
                    total_space_freed += space_freed;
                    orphaned_cleaned += 1;
                }
                _ => {
                    if args.verbose {
                        println!("{} {} (failed to delete)", "✗".red(), orphaned.display());
                    }
                }
            }
        }
    }

    // Clean node_modules directories
    for node_modules in &node_modules_to_clean {
        if args.dry_run {
            println!("{} {}", "[DRY RUN NODE_MODULES]".yellow(), node_modules.display());
        } else {
            let space_freed = cleaner::calculate_dir_size(node_modules).unwrap_or(0);
            match cleaner::delete_node_modules_dir(node_modules, args.dry_run) {
                Ok(Some(_)) => {
                    println!("{} {}", "📦".cyan(), node_modules.display());
                    logger.log_node_modules_cleaned(&node_modules.display().to_string(), space_freed)?;
                    total_space_freed += space_freed;
                    node_modules_cleaned += 1;
                }
                _ => {
                    if args.verbose {
                        println!("{} {} (failed to delete)", "✗".red(), node_modules.display());
                    }
                }
            }
        }
    }

    // Clean Python virtual environments
    for venv in &venvs_to_clean {
        if args.dry_run {
            println!("{} {}", "[DRY RUN VENV]".yellow(), venv.display());
        } else {
            let space_freed = cleaner::calculate_dir_size(venv).unwrap_or(0);
            match cleaner::delete_venv_dir(venv, args.dry_run) {
                Ok(Some(_)) => {
                    println!("{} {}", "🐍".cyan(), venv.display());
                    logger.log_venv_cleaned(&venv.display().to_string(), space_freed)?;
                    total_space_freed += space_freed;
                    venvs_cleaned += 1;
                }
                _ => {
                    if args.verbose {
                        println!("{} {} (failed to delete)", "✗".red(), venv.display());
                    }
                }
            }
        }
    }

    // Clean sccache directories
    for sccache in &sccache_to_clean {
        if args.dry_run {
            println!("{} {}", "[DRY RUN SCCACHE]".yellow(), sccache.display());
        } else {
            let space_freed = cleaner::calculate_dir_size(sccache).unwrap_or(0);
            match cleaner::delete_sccache_dir(sccache, args.dry_run) {
                Ok(Some(_)) => {
                    println!("{} {}", "🔧".cyan(), sccache.display());
                    logger.log_sccache_cleaned(&sccache.display().to_string(), space_freed)?;
                    total_space_freed += space_freed;
                    sccache_cleaned += 1;
                }
                _ => {
                    if args.verbose {
                        println!("{} {} (failed to delete)", "✗".red(), sccache.display());
                    }
                }
            }
        }
    }

    // Clean Stack work directories
    for stack_work in &stack_work_to_clean {
        if args.dry_run {
            println!("{} {}", "[DRY RUN STACK-WORK]".yellow(), stack_work.display());
        } else {
            let space_freed = cleaner::calculate_dir_size(stack_work).unwrap_or(0);
            match cleaner::delete_stack_work_dir(stack_work, args.dry_run) {
                Ok(Some(_)) => {
                    println!("{} {}", "λ".cyan(), stack_work.display());
                    logger.log_stack_work_cleaned(&stack_work.display().to_string(), space_freed)?;
                    total_space_freed += space_freed;
                    stack_work_cleaned += 1;
                }
                _ => {
                    if args.verbose {
                        println!("{} {} (failed to delete)", "✗".red(), stack_work.display());
                    }
                }
            }
        }
    }

    // Clean rustup directories
    for rustup in &rustup_to_clean {
        if args.dry_run {
            println!("{} {}", "[DRY RUN RUSTUP]".yellow(), rustup.display());
        } else {
            let space_freed = cleaner::calculate_dir_size(rustup).unwrap_or(0);
            match cleaner::delete_rustup_dir(rustup, args.dry_run) {
                Ok(Some(_)) => {
                    println!("{} {}", "🦀".cyan(), rustup.display());
                    logger.log_rustup_cleaned(&rustup.display().to_string(), space_freed)?;
                    total_space_freed += space_freed;
                    rustup_cleaned += 1;
                }
                _ => {
                    if args.verbose {
                        println!("{} {} (failed to delete)", "✗".red(), rustup.display());
                    }
                }
            }
        }
    }

    // Clean Next.js build directories
    for next in &next_to_clean {
        if args.dry_run {
            println!("{} {}", "[DRY RUN NEXT]".yellow(), next.display());
        } else {
            let space_freed = cleaner::calculate_dir_size(next).unwrap_or(0);
            match cleaner::delete_next_dir(next, args.dry_run) {
                Ok(Some(_)) => {
                    println!("{} {}", "▲".cyan(), next.display());
                    logger.log_next_cleaned(&next.display().to_string(), space_freed)?;
                    total_space_freed += space_freed;
                    next_cleaned += 1;
                }
                _ => {
                    if args.verbose {
                        println!("{} {} (failed to delete)", "✗".red(), next.display());
                    }
                }
            }
        }
    }

    // Clean cargo-nix directories
    for cargo_nix in &cargo_nix_to_clean {
        if args.dry_run {
            println!("{} {}", "[DRY RUN CARGO-NIX]".yellow(), cargo_nix.display());
        } else {
            let space_freed = cleaner::calculate_dir_size(cargo_nix).unwrap_or(0);
            match cleaner::delete_cargo_nix_dir(cargo_nix, args.dry_run) {
                Ok(Some(_)) => {
                    println!("{} {}", "❄".cyan(), cargo_nix.display());
                    logger.log_cargo_nix_cleaned(&cargo_nix.display().to_string(), space_freed)?;
                    total_space_freed += space_freed;
                    cargo_nix_cleaned += 1;
                }
                _ => {
                    if args.verbose {
                        println!("{} {} (failed to delete)", "✗".red(), cargo_nix.display());
                    }
                }
            }
        }
    }

    // Print summary
    println!(); // Empty line before summary
    let successful = results.iter().filter(|r| r.is_success()).count();
    let target_only = results.iter().filter(|r| r.is_target_only()).count();
    let skipped = results.iter().filter(|r| r.is_skipped()).count();
    let failed = results.len() - successful - target_only - skipped;

    if args.dry_run {
        let total_items = results.len() + orphaned_to_clean.len() + node_modules_to_clean.len() + venvs_to_clean.len() + sccache_to_clean.len() + stack_work_to_clean.len() + rustup_to_clean.len() + next_to_clean.len() + cargo_nix_to_clean.len();
        println!(
            "{} {} {} would be cleaned",
            "Summary:".bold(),
            total_items,
            if total_items == 1 { "item" } else { "items" }
        );
    } else {
        println!("{}", "Summary:".bold().green());

        if successful > 0 {
            println!(
                "         {} {} cleaned",
                successful,
                if successful == 1 { "Rust project" } else { "Rust projects" }
            );
        }

        if target_only > 0 {
            println!(
                "         {} {} cleaned (target only - invalid config)",
                target_only,
                if target_only == 1 { "Rust project" } else { "Rust projects" }
            );
        }

        if orphaned_cleaned > 0 {
            println!(
                "         {} {} cleaned",
                orphaned_cleaned,
                if orphaned_cleaned == 1 { "orphaned target" } else { "orphaned targets" }
            );
        }

        if node_modules_cleaned > 0 {
            println!(
                "         {} {}",
                node_modules_cleaned,
                if node_modules_cleaned == 1 { "node_modules" } else { "node_modules" }
            );
        }

        if venvs_cleaned > 0 {
            println!(
                "         {} {}",
                venvs_cleaned,
                if venvs_cleaned == 1 { "Python venv" } else { "Python venvs" }
            );
        }

        if sccache_cleaned > 0 {
            println!(
                "         {} {}",
                sccache_cleaned,
                if sccache_cleaned == 1 { "sccache directory" } else { "sccache directories" }
            );
        }

        if stack_work_cleaned > 0 {
            println!(
                "         {} {}",
                stack_work_cleaned,
                if stack_work_cleaned == 1 { "Stack work directory" } else { "Stack work directories" }
            );
        }

        if rustup_cleaned > 0 {
            println!(
                "         {} {}",
                rustup_cleaned,
                if rustup_cleaned == 1 { "rustup directory" } else { "rustup directories" }
            );
        }

        if next_cleaned > 0 {
            println!(
                "         {} {}",
                next_cleaned,
                if next_cleaned == 1 { "Next.js build" } else { "Next.js builds" }
            );
        }

        if cargo_nix_cleaned > 0 {
            println!(
                "         {} {}",
                cargo_nix_cleaned,
                if cargo_nix_cleaned == 1 { "cargo-nix directory" } else { "cargo-nix directories" }
            );
        }

        if total_space_freed > 0 {
            println!(
                "         {} total space freed",
                human_bytes(total_space_freed).bold().cyan()
            );
        }

        if skipped > 0 {
            println!(
                "         {} {} skipped (no target directory)",
                skipped,
                if skipped == 1 { "project" } else { "projects" }
            );
        }

        if failed > 0 {
            println!(
                "         {} {} failed",
                failed,
                if failed == 1 { "project" } else { "projects" }
            );
        }
    }

    // Log summary
    logger.log_summary(
        results.len(),
        successful,
        target_only,
        skipped,
        failed,
        orphaned_cleaned,
        node_modules_cleaned,
        venvs_cleaned,
        sccache_cleaned,
        stack_work_cleaned,
        rustup_cleaned,
        next_cleaned,
        cargo_nix_cleaned,
        total_space_freed,
    )?;

    // Print log file location
    println!();
    println!(
        "{} {}",
        "Log file:".dimmed(),
        logger.path().display().to_string().dimmed()
    );

    Ok(())
}

/// Converts bytes to human-readable format
fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
