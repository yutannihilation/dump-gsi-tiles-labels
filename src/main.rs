use std::error::Error;
use std::io::{Read, Seek};

use clap::Parser;
use directory::parse_root_directory;
use flate2::read::GzDecoder;
use nom::AsBytes;

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

    let mut root_dir_raw = vec![0u8; header.root_directory_length as usize];
    file.seek(std::io::SeekFrom::Start(header.root_directory_offset))?;
    file.read_exact(&mut root_dir_raw)?;

    let mut root_dir_decoded = vec![];
    let mut decoded_reader = GzDecoder::new(root_dir_raw.as_bytes());
    decoded_reader.read_to_end(&mut root_dir_decoded)?;

    let (rest, entries) =
        parse_root_directory(&root_dir_decoded).expect("Failed to parse root directory");
    for e in &entries {
        println!("{e:?}");
    }
    debug_assert!(rest.is_empty());

    // for b in root_dir_decoded {
    //     println!("{b:08b} ({b})");
    // }

    // let (remaining, header) = header::parse_pmtiles(&root_dir_data).expect("Failed to parse");
    // println!("{header:#?}");

    Ok(())
}
