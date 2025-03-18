use file_deduplicator::add;
use std::io;

fn main() -> io::Result<()> {
    println!("1 + 2 = {}", add(1, 2));
    Ok(())
}
