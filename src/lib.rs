use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<PathBuf>,
}

pub fn hash_file(path: &Path) -> Result<String> {
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn is_subdirectory(parent: &Path, child: &Path) -> bool {
    let child = child.canonicalize().unwrap();
    let parent = parent.canonicalize().unwrap();
    child.starts_with(parent)
}

pub fn find_duplicates<F>(
    dir1: &Path,
    dir2: &Path,
    progress_callback: F,
) -> Result<HashMap<String, Vec<PathBuf>>>
where
    F: Fn(usize, usize, usize, &str) + Send + Sync,
{
    let mut files = Vec::new();
    let mut file_count = 0;

    let dir1_canonical = dir1.canonicalize()?;
    let dir2_canonical = dir2.canonicalize()?;

    let scanning_from = if is_subdirectory(&dir1_canonical, &dir2_canonical) {
        &dir1_canonical
    } else if is_subdirectory(&dir2_canonical, &dir1_canonical) {
        &dir2_canonical
    } else {
        for dir in [&dir1_canonical, &dir2_canonical] {
            for entry in WalkDir::new(dir)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    files.push(entry.path().to_path_buf());
                    file_count += 1;
                    progress_callback(file_count, 0, 0, "Scanning directories");
                }
            }
        }
        return process_files(files, progress_callback);
    };

    for entry in WalkDir::new(scanning_from)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
            file_count += 1;
            progress_callback(file_count, 0, 0, "Scanning directories");
        }
    }

    process_files(files, progress_callback)
}

fn process_files<F>(
    files: Vec<PathBuf>,
    progress_callback: F,
) -> Result<HashMap<String, Vec<PathBuf>>>
where
    F: Fn(usize, usize, usize, &str) + Send + Sync,
{
    let total_files = files.len();
    let hash_map: HashMap<String, Vec<PathBuf>> = files
        .par_iter()
        .enumerate()
        .filter_map(|(idx, path)| {
            if let Ok(hash) = hash_file(path) {
                progress_callback(idx + 1, total_files, 0, "Processing files");
                Some((hash, path.clone()))
            } else {
                None
            }
        })
        .fold(
            || HashMap::<String, Vec<PathBuf>>::new(),
            |mut acc, (hash, path)| {
                acc.entry(hash).or_default().push(path);
                acc
            },
        )
        .reduce(
            || HashMap::new(),
            |mut map1, map2| {
                for (hash, paths) in map2 {
                    map1.entry(hash).or_default().extend(paths);
                }
                map1
            },
        );

    Ok(hash_map)
}

pub fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} bytes", size)
    }
}
