use anyhow::{Context, Result};
use clap::Parser;
use file_deduplicator::find_duplicates;
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

    let duplicates =
        find_duplicates(&args.dir1, &args.dir2).context("Failed to find duplicates")?;

    let mut total_wasted_space = 0;
    for path in &duplicates {
        if let Ok(metadata) = fs::metadata(path) {
            total_wasted_space += metadata.len();
            println!("{}", path.display());
        }
    }

    eprintln!(
        "Total wasted space: {}",
        file_deduplicator::format_size(total_wasted_space)
    );

    Ok(())
}
