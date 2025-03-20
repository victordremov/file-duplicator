use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_find_duplicates_with_same_directory() -> io::Result<()> {
    let temp_dir1 = tempdir()?;

    // Create test file in first directory
    let file_path1 = temp_dir1.path().join("test_file.txt");
    let mut file1 = File::create(&file_path1)?;
    writeln!(file1, "This is a test file for deduplication testing")?;
    file1.flush()?;

    // Create a duplicate file
    let dup_file_path = temp_dir1.path().join("duplicate_file.txt");
    fs::copy(&file_path1, &dup_file_path)?;

    // Make a subdirectory with another duplicate
    let subdir_path = temp_dir1.path().join("subdir");
    fs::create_dir(&subdir_path)?;
    let nested_dup_path = subdir_path.join("nested_duplicate.txt");
    fs::copy(&file_path1, &nested_dup_path)?;

    // Run the deduplicator
    let output = Command::new(env!("CARGO_BIN_EXE_file-deduplicator"))
        .arg(temp_dir1.path())
        .arg(temp_dir1.path())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command failed: {}", stderr);
    assert!(
        stdout
            .lines()
            .filter(|line| {
                line.contains("duplicate_file.txt")
                    || line.contains("nested_duplicate.txt")
                    || line.contains("test_file.txt")
            })
            .count()
            == 2,
        "Expected at least 2 duplicate files in output"
    );
    assert!(stderr.contains("Total wasted space:"));
    Ok(())
}

#[test]
fn test_find_duplicates_across_directories() -> io::Result<()> {
    // Create two temporary directories
    let temp_dir1 = tempdir()?;
    let temp_dir2 = tempdir()?;

    // Create test file in first directory
    let file_path1 = temp_dir1.path().join("original.txt");
    let mut file1 = File::create(&file_path1)?;
    writeln!(file1, "Content to be duplicated across directories")?;
    file1.flush()?;

    // Create identical file in second directory with different name
    let file_path2 = temp_dir2.path().join("duplicate.txt");
    fs::copy(&file_path1, &file_path2)?;

    // Create a different file in second directory
    let unique_path = temp_dir2.path().join("unique.txt");
    let mut unique_file = File::create(&unique_path)?;
    writeln!(
        unique_file,
        "This file is unique and should not be detected as duplicate"
    )?;
    unique_file.flush()?;

    // Run the deduplicator between the two directories
    let output = Command::new(env!("CARGO_BIN_EXE_file-deduplicator"))
        .arg(temp_dir1.path())
        .arg(temp_dir2.path())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    // Verify success
    assert!(output.status.success());

    // Verify duplicate is found
    assert!(stdout.contains("duplicate.txt"));

    // Verify the unique file is not included
    assert!(!stdout.contains("unique.txt"));

    // Verify wasted space is reported
    assert!(stderr.contains("Total wasted space:"));

    Ok(())
}

#[test]
fn test_no_duplicates() -> io::Result<()> {
    // Create two temporary directories
    let temp_dir1 = tempdir()?;
    let temp_dir2 = tempdir()?;

    // Create different files in each directory
    let file_path1 = temp_dir1.path().join("file1.txt");
    let mut file1 = File::create(&file_path1)?;
    writeln!(file1, "Content for file 1")?;
    file1.flush()?;

    let file_path2 = temp_dir2.path().join("file2.txt");
    let mut file2 = File::create(&file_path2)?;
    writeln!(file2, "Completely different content for file 2")?;
    file2.flush()?;

    // Run the deduplicator
    let output = Command::new(env!("CARGO_BIN_EXE_file-deduplicator"))
        .arg(temp_dir1.path())
        .arg(temp_dir2.path())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify success
    assert!(output.status.success());

    // Verify output is empty (no duplicates found)
    assert!(stdout.is_empty());
    // Should still show total (which would be 0)
    assert!(stderr.contains("Total wasted space:"));

    Ok(())
}
