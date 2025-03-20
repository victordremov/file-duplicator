use anyhow::{Context, Result};
use clap::Parser;
use file_deduplicator::{DuplicateGroup, find_duplicates, format_size};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about = "Find duplicate files between two directories"
)]
struct Args {
    #[arg(value_name = "DIR1")]
    dir1: PathBuf,

    #[arg(value_name = "DIR2")]
    dir2: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

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

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let duplicate_groups = find_duplicates(
        &args.dir1,
        &args.dir2,
        |current, total, dup_count, stage| {
            if total == 0 {
                // During directory scanning
                pb.set_message(format!("{}: found {} files", stage, current));
            } else {
                // During file processing
                let percentage = (current as f64 / total as f64 * 100.0) as u64;
                pb.set_position(percentage);
                pb.set_message(format!(
                    "{}: {}/{} files ({}%) - {} duplicates",
                    stage, current, total, percentage, dup_count
                ));
            }
        },
    )
    .context("Failed to find duplicates")?;

    pb.finish_and_clear();

    let mut groups: Vec<DuplicateGroup> = Vec::new();
    let mut total_wasted_space = 0;

    for (hash, paths) in duplicate_groups {
        if paths.len() > 1 {
            if let Ok(size) = fs::metadata(&paths[0]).map(|m| m.len()) {
                total_wasted_space += size * (paths.len() as u64 - 1);
                groups.push(DuplicateGroup {
                    hash,
                    size,
                    files: paths,
                });
            }
        }
    }

    println!("{}", serde_json::to_string_pretty(&groups)?);
    eprintln!("Total wasted space: {}", format_size(total_wasted_space));

    Ok(())
}
