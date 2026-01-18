//! Cache management commands

use anyhow::Result;
use colored::Colorize;
use std::process::ExitCode;

use crate::cache::CacheManager;

/// Clear all cache entries
pub fn clear() -> Result<ExitCode> {
    let cache_mgr = CacheManager::new()?;

    println!("{}", "Clearing generation cache...".cyan().bold());

    let count = cache_mgr.clear()?;

    if count == 0 {
        println!("  {}", "Cache is already empty".dimmed());
    } else {
        println!(
            "  {} Removed {} cache {}",
            "SUCCESS".green().bold(),
            count,
            if count == 1 { "entry" } else { "entries" }
        );
    }

    Ok(ExitCode::SUCCESS)
}

/// Show cache information
pub fn info() -> Result<ExitCode> {
    let cache_mgr = CacheManager::new()?;

    println!("{}", "Cache Information".cyan().bold());

    let info = cache_mgr.info()?;

    println!(
        "  {}: {}",
        "Cache directory".dimmed(),
        info.cache_dir.display()
    );
    println!("  {}: {}", "Entry count".dimmed(), info.entry_count);

    let size_mb = info.total_size_bytes as f64 / (1024.0 * 1024.0);
    if size_mb >= 1.0 {
        println!("  {}: {:.2} MB", "Total size".dimmed(), size_mb);
    } else {
        let size_kb = info.total_size_bytes as f64 / 1024.0;
        println!("  {}: {:.2} KB", "Total size".dimmed(), size_kb);
    }

    Ok(ExitCode::SUCCESS)
}
