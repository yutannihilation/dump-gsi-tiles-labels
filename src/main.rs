use std::error::Error;
use std::io::Read;

use clap::Parser;

mod header;
mod util;

#[derive(Parser, Debug)]
struct Args {
    file: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut file = std::fs::File::open(args.file)?;
    let mut data = vec![0u8; header::HEADER_BYTES];
    file.read_exact(&mut data)?;

    let (remaining, result) = header::parse_pmtiles(&data).expect("Failed to parse");

    assert!(remaining.is_empty());

    println!("{result:#?}");

    Ok(())
}
