use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

pub fn find_duplicates(dir1: &Path, dir2: &Path) -> Result<Vec<PathBuf>> {
    let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut duplicates = Vec::new();

    for file_path in find_files(dir1)? {
        let hash = hash_file(&file_path)?;
        hash_map.entry(hash).or_default().push(file_path);
    }

    for file_path in find_files(dir2)? {
        let hash = hash_file(&file_path)?;
        if hash_map.contains_key(&hash) && (hash_map[&hash][0] != file_path) {
            duplicates.push(file_path);
        }
    }

    Ok(duplicates)
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
