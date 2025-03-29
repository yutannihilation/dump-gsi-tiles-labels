use std::error::Error;
use std::io::Read as _;

use clap::{Parser, Subcommand};
use directory::parse_root_directory;
use header::PMTilesHeaderV3;

mod directory;
mod header;
mod util;
mod varint;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ShowHeader { file: std::path::PathBuf },
    ShowMetadata { file: std::path::PathBuf },
    List { file: std::path::PathBuf },
}

fn show_header(header: &PMTilesHeaderV3) {
    println!("{header:#?}");
}

fn show_metadata(file: &mut std::fs::File, header: &PMTilesHeaderV3) -> Result<(), Box<dyn Error>> {
    let metadata_decoded = util::decompress(
        file,
        header.metadata_offset,
        header.metadata_length as usize,
        &header.internal_compression,
    )?;

    println!("metadata: {}", String::from_utf8(metadata_decoded)?);
    Ok(())
}

fn list_entries(file: &mut std::fs::File, header: &PMTilesHeaderV3) -> Result<(), Box<dyn Error>> {
    let root_dir_decoded = util::decompress(
        file,
        header.root_directory_offset,
        header.root_directory_length as usize,
        &header.internal_compression,
    )?;

    let leaf_dir_decoded = util::decompress(
        file,
        header.leaf_directories_offset,
        header.leaf_directories_length as usize,
        &header.internal_compression,
    )?;

    let (rest, entries) =
        parse_root_directory(&root_dir_decoded).expect("Failed to parse root directory");
    for e in &entries {
        println!("{e:?}");
    }
    debug_assert!(rest.is_empty());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let mut file = match &args.command {
        Commands::ShowHeader { file } => std::fs::File::open(file)?,
        Commands::ShowMetadata { file } => std::fs::File::open(file)?,
        Commands::List { file } => std::fs::File::open(file)?,
    };

    // read header

    let mut header_data = vec![0u8; header::HEADER_BYTES];
    file.read_exact(&mut header_data)?;
    let (rest, header) = header::parse_header(&header_data).expect("Failed to parse haeder");
    debug_assert!(rest.is_empty());

    match &args.command {
        Commands::ShowHeader { .. } => show_header(&header),
        Commands::ShowMetadata { .. } => show_metadata(&mut file, &header)?,
        Commands::List { .. } => list_entries(&mut file, &header)?,
    };

    Ok(())
}
