use std::io::Read as _;
use std::{collections::HashMap, error::Error};

use clap::{Parser, Subcommand};
use directory::parse_directory;
use header::PMTilesHeaderV3;
use prost::Message;

mod directory;
mod header;
mod util;
mod varint;

mod mvt {
    include!(concat!(env!("OUT_DIR"), "/vector_tile.rs"));
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ShowHeader {
        file: std::path::PathBuf,
    },
    ShowMetadata {
        file: std::path::PathBuf,
    },
    List {
        file: std::path::PathBuf,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Tile {
        file: std::path::PathBuf,
        offset: u64,
        length: usize,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Text {
        file: std::path::PathBuf,
        #[arg(long)]
        limit: Option<usize>,
    },
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

fn list_entries(
    file: &mut std::fs::File,
    header: &PMTilesHeaderV3,
    limit: usize,
) -> Result<(), Box<dyn Error>> {
    let root_dir_decoded = util::decompress(
        file,
        header.root_directory_offset,
        header.root_directory_length as usize,
        &header.internal_compression,
    )?;

    let (rest, entries) =
        parse_directory(&root_dir_decoded).expect("Failed to parse root directory");
    debug_assert!(rest.is_empty());

    for e in entries.iter().take(limit) {
        println!("{e:?}");

        if !e.is_tile {
            let leaf_dir_decoded = util::decompress(
                file,
                header.leaf_directories_offset + e.offset,
                e.length as usize,
                &header.internal_compression,
            )?;

            let (rest, leaf_entries) =
                parse_directory(&leaf_dir_decoded).expect("Failed to parse leaf directory");
            debug_assert!(rest.is_empty());

            for le in leaf_entries.iter().take(limit) {
                println!("└── {le:?}");
            }
            if leaf_entries.len() > limit {
                println!("    ...");
            }
        }
    }

    if entries.len() > limit {
        println!("...");
    }

    Ok(())
}

fn dump_single_tile(
    file: &mut std::fs::File,
    header: &PMTilesHeaderV3,
    offset: u64,
    length: usize,
    limit: usize,
) -> Result<(), Box<dyn Error>> {
    let tile_decoded = util::decompress(
        file,
        header.tile_data_offset + offset,
        length,
        &header.tile_compression,
    )?;
    let tile = mvt::Tile::decode(tile_decoded.as_slice())?;

    for layer in tile.layers.iter().take(limit) {
        println!("---------------------------------------------------");

        println!("name: {}", layer.name);

        println!("features:");
        for feature in layer.features.iter().take(limit) {
            println!("  - id: {:?}, type: {:?}", feature.id, feature.r#type());
        }
        if layer.features.len() > limit {
            println!("    ...");
        }

        println!("keys: {:?}", layer.keys);

        print!("values: [");
        for value in layer.values.iter().take(limit) {
            if let Some(v) = value.bool_value {
                print!("{v}, ");
            } else if let Some(v) = value.double_value {
                print!("{v}, ");
            } else if let Some(v) = value.float_value {
                print!("{v}, ");
            } else if let Some(v) = value.int_value {
                print!("{v}, ");
            } else if let Some(v) = value.sint_value {
                print!("{v}, ");
            } else if let Some(v) = value.uint_value {
                print!("{v}, ");
            } else if let Some(v) = &value.string_value {
                print!(r#""{v}", "#);
            } else {
                print!("(null)")
            }
        }
        if layer.values.len() > limit {
            print!("...");
        }
        println!("]");
    }

    Ok(())
}

fn dump_text(
    file: &mut std::fs::File,
    header: &PMTilesHeaderV3,
    limit: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    let root_dir_decoded = util::decompress(
        file,
        header.root_directory_offset,
        header.root_directory_length as usize,
        &header.internal_compression,
    )?;

    let (rest, entries) =
        parse_directory(&root_dir_decoded).expect("Failed to parse root directory");
    debug_assert!(rest.is_empty());

    let mut result: HashMap<String, usize> = HashMap::new();

    for e in &entries {
        if e.is_tile {
            let tile_decoded = util::decompress(
                file,
                header.tile_data_offset + e.offset,
                e.length as usize,
                &header.tile_compression,
            )?;
            let tile = mvt::Tile::decode(tile_decoded.as_slice())?;

            for l in tile.layers {
                for v in l.values {
                    match v.string_value {
                        Some(s) => {
                            let count = result.entry(s).or_insert(0);
                            *count += 1;
                        }
                        None => {}
                    }
                }
            }
        } else {
            let leaf_dir_decoded = util::decompress(
                file,
                header.leaf_directories_offset + e.offset,
                e.length as usize,
                &header.internal_compression,
            )?;

            let (rest, leaf_entries) =
                parse_directory(&leaf_dir_decoded).expect("Failed to parse leaf directory");
            debug_assert!(rest.is_empty());

            for le in &leaf_entries {
                // TODO
            }
        }
    }

    println!("{result:#?}");

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let mut file = match &args.command {
        Commands::ShowHeader { file } => std::fs::File::open(file)?,
        Commands::ShowMetadata { file } => std::fs::File::open(file)?,
        Commands::List { file, .. } => std::fs::File::open(file)?,
        Commands::Tile { file, .. } => std::fs::File::open(file)?,
        Commands::Text { file, .. } => std::fs::File::open(file)?,
    };

    // read header

    let mut header_data = vec![0u8; header::HEADER_BYTES];
    file.read_exact(&mut header_data)?;
    let (rest, header) = header::parse_header(&header_data).expect("Failed to parse haeder");
    debug_assert!(rest.is_empty());

    match &args.command {
        Commands::ShowHeader { .. } => show_header(&header),
        Commands::ShowMetadata { .. } => show_metadata(&mut file, &header)?,
        Commands::List { limit, .. } => list_entries(&mut file, &header, *limit)?,
        Commands::Tile {
            offset,
            length,
            limit,
            ..
        } => dump_single_tile(&mut file, &header, *offset, *length, *limit)?,
        Commands::Text { limit, .. } => dump_text(&mut file, &header, *limit)?,
    };

    Ok(())
}
