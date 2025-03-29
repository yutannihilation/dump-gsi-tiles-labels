use std::error::Error;
use std::io::Read as _;

use clap::Parser;
use directory::parse_root_directory;

mod directory;
mod header;
mod util;
mod varint;

#[derive(Parser, Debug)]
struct Args {
    file: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut file = std::fs::File::open(args.file)?;

    // read header

    let mut header_data = vec![0u8; header::HEADER_BYTES];
    file.read_exact(&mut header_data)?;
    let (rest, header) = header::parse_header(&header_data).expect("Failed to parse haeder");
    println!("{header:#?}");
    debug_assert!(rest.is_empty());

    // read root dir

    let root_dir_decoded = util::decompress(
        &mut file,
        header.root_directory_offset,
        header.root_directory_length as usize,
        &header.internal_compression,
    )?;

    let (rest, entries) =
        parse_root_directory(&root_dir_decoded).expect("Failed to parse root directory");
    for e in &entries {
        println!("{e:?}");
    }
    debug_assert!(rest.is_empty());

    // read metadata

    let metadata_decoded = util::decompress(
        &mut file,
        header.metadata_offset,
        header.metadata_length as usize,
        &header.internal_compression,
    )?;

    println!("metadata: {}", String::from_utf8(metadata_decoded)?);

    // read leaf dir

    let root_dir_decoded = util::decompress(
        &mut file,
        header.root_directory_offset,
        header.root_directory_length as usize,
        &header.internal_compression,
    )?;

    Ok(())
}
