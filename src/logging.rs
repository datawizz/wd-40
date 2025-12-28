use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Logger {
    file: File,
    log_path: PathBuf,
}

impl Logger {
    /// Creates a new logger, either at the specified path or in the default cache directory
    pub fn new(custom_path: Option<PathBuf>) -> Result<Self> {
        let log_path = if let Some(path) = custom_path {
            path
        } else {
            // Default: ~/.cache/wd-40/clean-<timestamp>.log
            let cache_dir = dirs::cache_dir()
                .context("Failed to determine cache directory")?
                .join("wd-40");

            fs::create_dir_all(&cache_dir)
                .context("Failed to create cache directory")?;

            let timestamp = Local::now().format("%Y%m%d-%H%M%S");
            cache_dir.join(format!("clean-{}.log", timestamp))
        };

        let file = File::create(&log_path)
            .with_context(|| format!("Failed to create log file: {}", log_path.display()))?;

        let mut logger = Logger { file, log_path };
        logger.write_header()?;
        Ok(logger)
    }

    /// Returns the path to the log file
    pub fn path(&self) -> &Path {
        &self.log_path
    }

    fn write_header(&mut self) -> Result<()> {
        writeln!(self.file, "WD-40 Rust Project Cleaner")?;
        writeln!(self.file, "==========================")?;
        writeln!(self.file, "Started: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_projects(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} projects:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_orphaned(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} orphaned target directories:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_node_modules(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} node_modules directories:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_venvs(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} Python virtual environments:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_sccache(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} sccache directories:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_stack_work(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} Stack work directories:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_rustup(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} rustup directories:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_next(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} Next.js build directories:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_found_cargo_nix(&mut self, count: usize, paths: &[PathBuf]) -> Result<()> {
        writeln!(self.file, "Found {} cargo-nix directories:", count)?;
        for path in paths {
            writeln!(self.file, "  - {}", path.display())?;
        }
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_cleaning_start(&mut self) -> Result<()> {
        writeln!(self.file, "Starting cleanup...")?;
        writeln!(self.file)?;
        Ok(())
    }

    pub fn log_success(&mut self, project: &str, space_freed: Option<u64>) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        if let Some(bytes) = space_freed {
            writeln!(
                self.file,
                "[{}] SUCCESS: {} (freed {})",
                timestamp,
                project,
                human_bytes(bytes)
            )?;
        } else {
            writeln!(self.file, "[{}] SUCCESS: {}", timestamp, project)?;
        }
        Ok(())
    }

    pub fn log_skipped(&mut self, project: &str, reason: &str) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(self.file, "[{}] SKIPPED: {} - {}", timestamp, project, reason)?;
        Ok(())
    }

    pub fn log_failed(&mut self, project: &str, error: &str) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(self.file, "[{}] FAILED: {} - {}", timestamp, project, error)?;
        Ok(())
    }

    pub fn log_target_only(&mut self, project: &str, space_freed: u64, reason: &str) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] TARGET ONLY: {} (freed {}) - {}",
            timestamp,
            project,
            human_bytes(space_freed),
            reason
        )?;
        Ok(())
    }

    pub fn log_orphaned_cleaned(&mut self, target_path: &str, space_freed: u64) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] ORPHANED: {} (freed {})",
            timestamp,
            target_path,
            human_bytes(space_freed)
        )?;
        Ok(())
    }

    pub fn log_node_modules_cleaned(&mut self, path: &str, space_freed: u64) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] NODE_MODULES: {} (freed {})",
            timestamp,
            path,
            human_bytes(space_freed)
        )?;
        Ok(())
    }

    pub fn log_venv_cleaned(&mut self, path: &str, space_freed: u64) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] PYTHON_VENV: {} (freed {})",
            timestamp,
            path,
            human_bytes(space_freed)
        )?;
        Ok(())
    }

    pub fn log_sccache_cleaned(&mut self, path: &str, space_freed: u64) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] SCCACHE: {} (freed {})",
            timestamp,
            path,
            human_bytes(space_freed)
        )?;
        Ok(())
    }

    pub fn log_stack_work_cleaned(&mut self, path: &str, space_freed: u64) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] STACK_WORK: {} (freed {})",
            timestamp,
            path,
            human_bytes(space_freed)
        )?;
        Ok(())
    }

    pub fn log_rustup_cleaned(&mut self, path: &str, space_freed: u64) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] RUSTUP: {} (freed {})",
            timestamp,
            path,
            human_bytes(space_freed)
        )?;
        Ok(())
    }

    pub fn log_next_cleaned(&mut self, path: &str, space_freed: u64) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] NEXT: {} (freed {})",
            timestamp,
            path,
            human_bytes(space_freed)
        )?;
        Ok(())
    }

    pub fn log_cargo_nix_cleaned(&mut self, path: &str, space_freed: u64) -> Result<()> {
        let timestamp = Local::now().format("%H:%M:%S");
        writeln!(
            self.file,
            "[{}] CARGO_NIX: {} (freed {})",
            timestamp,
            path,
            human_bytes(space_freed)
        )?;
        Ok(())
    }

    pub fn log_summary(
        &mut self,
        total: usize,
        successful: usize,
        target_only: usize,
        skipped: usize,
        failed: usize,
        orphaned_cleaned: usize,
        node_modules_cleaned: usize,
        venvs_cleaned: usize,
        sccache_cleaned: usize,
        stack_work_cleaned: usize,
        rustup_cleaned: usize,
        next_cleaned: usize,
        cargo_nix_cleaned: usize,
        total_space_freed: u64,
    ) -> Result<()> {
        writeln!(self.file)?;
        writeln!(self.file, "==========================")?;
        writeln!(self.file, "Summary")?;
        writeln!(self.file, "==========================")?;
        writeln!(self.file, "Total projects found: {}", total)?;
        writeln!(self.file, "Successfully cleaned: {}", successful)?;
        writeln!(self.file, "Target-only cleaned: {}", target_only)?;
        writeln!(self.file, "Skipped: {}", skipped)?;
        writeln!(self.file, "Failed: {}", failed)?;
        writeln!(self.file, "Orphaned targets cleaned: {}", orphaned_cleaned)?;
        writeln!(self.file, "Node modules cleaned: {}", node_modules_cleaned)?;
        writeln!(self.file, "Python venvs cleaned: {}", venvs_cleaned)?;
        writeln!(self.file, "Sccache dirs cleaned: {}", sccache_cleaned)?;
        writeln!(self.file, "Stack work dirs cleaned: {}", stack_work_cleaned)?;
        writeln!(self.file, "Rustup dirs cleaned: {}", rustup_cleaned)?;
        writeln!(self.file, "Next.js builds cleaned: {}", next_cleaned)?;
        writeln!(self.file, "Cargo-nix dirs cleaned: {}", cargo_nix_cleaned)?;
        writeln!(self.file, "Total space freed: {}", human_bytes(total_space_freed))?;
        writeln!(self.file)?;
        writeln!(self.file, "Completed: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_bytes() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1024), "1.00 KB");
        assert_eq!(human_bytes(1536), "1.50 KB");
        assert_eq!(human_bytes(1048576), "1.00 MB");
        assert_eq!(human_bytes(1073741824), "1.00 GB");
    }
}
