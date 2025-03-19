use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a duplicate file
#[derive(Debug)]
pub struct DuplicateFile {
    pub hash: String,
    pub paths: Vec<PathBuf>,
    pub size: u64,
}

/// Calculate SHA-256 hash for a file
pub fn hash_file(path: &Path) -> Result<String> {
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// Find all files in a directory recursively
pub fn find_files(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}

/// Find duplicate files between two directories
pub fn find_duplicates(dir1: &Path, dir2: &Path) -> Result<Vec<DuplicateFile>> {
    // Step 1: Index files by size first (quick filter)
    let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    // Process first directory
    for file_path in find_files(dir1)? {
        let metadata = fs::metadata(&file_path)
            .with_context(|| format!("Failed to get metadata for: {}", file_path.display()))?;
        size_map.entry(metadata.len()).or_default().push(file_path);
    }

    // Process second directory and track potential duplicates
    let mut potential_duplicates: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for file_path in find_files(dir2)? {
        let metadata = fs::metadata(&file_path)
            .with_context(|| format!("Failed to get metadata for: {}", file_path.display()))?;
        let size = metadata.len();

        if size_map.contains_key(&size) {
            // First add existing files from dir1 if we haven't yet
            if !potential_duplicates.contains_key(&size) {
                potential_duplicates.insert(size, size_map[&size].clone());
            }
            // Then add this file from dir2
            potential_duplicates.get_mut(&size).unwrap().push(file_path);
        }
    }

    // Step 2: Hash files with same size to confirm duplicates
    let mut duplicates = Vec::new();
    let mut processed_hashes = HashSet::new();

    for (size, paths) in potential_duplicates {
        if paths.len() < 2 {
            continue; // Need at least 2 files to have a duplicate
        }

        let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for path in paths {
            let hash = hash_file(&path)?;
            hash_map.entry(hash).or_default().push(path);
        }

        for (hash, paths) in hash_map {
            if paths.len() >= 2 && !processed_hashes.contains(&hash) {
                duplicates.push(DuplicateFile {
                    hash: hash.clone(),
                    paths: paths.clone(),
                    size,
                });
                processed_hashes.insert(hash);
            }
        }
    }

    Ok(duplicates)
}

/// Format file size in human-readable format
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
