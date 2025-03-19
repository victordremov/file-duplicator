use anyhow::{Context, Result};
use clap::Parser;
use file_deduplicator::find_duplicates;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

/// CLI arguments
#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about = "Find duplicate files between two directories"
)]
struct Args {
    /// First directory to scan
    #[arg(value_name = "DIR1")]
    dir1: PathBuf,

    /// Second directory to scan
    #[arg(value_name = "DIR2")]
    dir2: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate directories
    if !args.dir1.is_dir() {
        return Err(anyhow::anyhow!(
            "{} is not a directory",
            args.dir1.display()
        ));
    }
    if !args.dir2.is_dir() {
        return Err(anyhow::anyhow!(
            "{} is not a directory",
            args.dir2.display()
        ));
    }

    println!("Scanning for duplicate files...");
    println!("Directory 1: {}", args.dir1.display());
    println!("Directory 2: {}", args.dir2.display());

    // Set up progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Finding duplicates...");

    // Find duplicates
    let duplicates =
        find_duplicates(&args.dir1, &args.dir2).context("Failed to find duplicates")?;

    pb.finish_with_message(format!("Found {} duplicate sets", duplicates.len()));

    // Display results
    if duplicates.is_empty() {
        println!("\nNo duplicate files found.");
    } else {
        println!("\nFound {} sets of duplicate files:", duplicates.len());

        let mut total_wasted_space = 0;

        for (i, dup) in duplicates.iter().enumerate() {
            let wasted_space = dup.size * (dup.paths.len() as u64 - 1);
            total_wasted_space += wasted_space;

            println!(
                "\nDuplicate set #{} ({})",
                i + 1,
                file_deduplicator::format_size(dup.size)
            );
            println!("SHA-256: {}", dup.hash);
            for path in &dup.paths {
                println!("  - {}", path.display());
            }
            println!(
                "Wasted space: {}",
                file_deduplicator::format_size(wasted_space)
            );
        }

        println!(
            "\nTotal wasted space: {}",
            file_deduplicator::format_size(total_wasted_space)
        );
    }

    Ok(())
}
