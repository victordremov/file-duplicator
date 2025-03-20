use criterion::{Criterion, criterion_group, criterion_main};
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

fn create_test_dirs() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();

    for i in 0..100 {
        let dir_path = temp_dir.path().join(format!("dir_{}", i));
        fs::create_dir(&dir_path).unwrap();

        for j in 0..100 {
            let file_path = dir_path.join(format!("file_{}.txt", j));
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "Content for file {} in dir {}", j, i).unwrap();
        }
    }

    // Create some duplicate files
    let original = temp_dir.path().join("dir_0/file_0.txt");
    let duplicate1 = temp_dir.path().join("dir_5/file_50.txt");
    let duplicate2 = temp_dir.path().join("dir_9/file_99.txt");

    fs::copy(&original, &duplicate1).unwrap();
    fs::copy(&original, &duplicate2).unwrap();

    temp_dir
}

fn find_duplicates_benchmark(c: &mut Criterion) {
    let temp_dir = create_test_dirs();

    let mut group = c.benchmark_group("find_duplicates");
    group.sample_size(40);

    group.bench_function("find_duplicates_1k_files", |b| {
        b.iter(|| {
            file_deduplicator::find_duplicates(temp_dir.path(), temp_dir.path(), |_, _, _, _| {})
                .unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, find_duplicates_benchmark);
criterion_main!(benches);
