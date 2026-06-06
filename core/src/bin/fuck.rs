use std::io::Write;
use unrar::{Archive, FileHeader, ListSplit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic args parsing
    // Usage: cargo run --example lister path/to/archive.rar
    let mut args = std::env::args();
    let mut stderr = std::io::stderr();

    let Some(file) = args.nth(1) else {
        writeln!(&mut stderr, "Please pass an archive as argument!")?;
        std::process::exit(1)
    };

    let mut archive = Archive::new(&file).open_for_processing()?;

    let mut headers = Vec::new();

    while let Some(header) = archive.read_header()? {
        println!(
            "{} bytes: {}",
            header.entry().unpacked_size,
            header.entry().filename.display(),
        );
        if header.entry().is_file() {
        
        archive = header.skip()?;
    }
    Ok(())
}
