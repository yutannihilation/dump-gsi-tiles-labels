use std::{collections::HashMap, error::Error};

use clap::{Parser, Subcommand};
use util::PMTilesFile;

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

fn show_header(file: &PMTilesFile) {
    println!("{:#?}", file.parse_header());
}

fn show_metadata(file: &mut PMTilesFile) -> Result<(), Box<dyn Error>> {
    println!("{}", file.parse_metadata()?);
    Ok(())
}

fn list_entries(file: &mut PMTilesFile, limit: usize) -> Result<(), Box<dyn Error>> {
    let entries = file.parse_root_directory()?;

    for e in entries.iter().take(limit) {
        println!("{e:?}");

        if !e.is_tile {
            let leaf_entries = file.parse_leaf_directory(e.offset, e.length as usize)?;

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
    file: &mut PMTilesFile,
    offset: u64,
    length: usize,
    limit: usize,
) -> Result<(), Box<dyn Error>> {
    let tile_type = &file.parse_header().tile_type;
    if !matches!(tile_type, header::PMTilesTileType::Mvt) {
        println!("Unsupported tile type: {tile_type:?}");
        return Ok(());
    }

    let tile = file.parse_tile(offset, length)?;

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
            util::print_tile_value(value);
        }
        if layer.values.len() > limit {
            print!("...");
        }
        println!("]");
    }

    Ok(())
}

fn dump_text(file: &mut PMTilesFile, limit: Option<usize>) -> Result<(), Box<dyn Error>> {
    let limit = limit.unwrap_or(usize::MAX);

    let entries = file.parse_root_directory()?;

    let mut result: HashMap<String, usize> = HashMap::new();

    for e in entries.into_iter().take(limit) {
        let leaf_entries = if e.is_tile {
            vec![e]
        } else {
            // if the entry in the root directory points to a leaf directory, parse it
            file.parse_leaf_directory(e.offset, e.length as usize)?
        };

        for le in &leaf_entries {
            let tile = file.parse_tile(le.offset, le.length as usize)?;

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
        }
    }

    // show result

    let mut sorted: Vec<(String, usize)> = result.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1)); // reverse sort
    for (k, v) in &sorted {
        println!("{k}: {v} times");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let mut file = match &args.command {
        Commands::ShowHeader { file } => PMTilesFile::new(file)?,
        Commands::ShowMetadata { file } => PMTilesFile::new(file)?,
        Commands::List { file, .. } => PMTilesFile::new(file)?,
        Commands::Tile { file, .. } => PMTilesFile::new(file)?,
        Commands::Text { file, .. } => PMTilesFile::new(file)?,
    };

    match &args.command {
        Commands::ShowHeader { .. } => show_header(&file),
        Commands::ShowMetadata { .. } => show_metadata(&mut file)?,
        Commands::List { limit, .. } => list_entries(&mut file, *limit)?,
        Commands::Tile {
            offset,
            length,
            limit,
            ..
        } => dump_single_tile(&mut file, *offset, *length, *limit)?,
        Commands::Text { limit, .. } => dump_text(&mut file, *limit)?,
    };

    Ok(())
}
