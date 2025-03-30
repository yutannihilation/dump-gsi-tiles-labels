use std::{
    error::Error,
    io::{Read as _, Seek as _},
};

use flate2::read::GzDecoder;
use nom::error::ErrorKind;
use prost::Message as _;

use crate::{
    directory::{self, PMTilesEntry},
    header::{PMTilesCompression, PMTilesHeaderV3},
    mvt,
};

pub(crate) fn nom_error<T>(input: &[u8]) -> Result<(&[u8], T), nom::Err<nom::error::Error<&[u8]>>> {
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        ErrorKind::Fail,
    )))
}

pub struct PMTilesFile {
    file: std::fs::File,
    header: PMTilesHeaderV3,
}

impl PMTilesFile {
    pub fn new<T: AsRef<std::path::Path>>(file: T) -> Result<Self, Box<dyn Error>> {
        let mut file = std::fs::File::open(file.as_ref())?;
        let mut header_data = vec![0u8; crate::header::HEADER_BYTES];
        file.read_exact(&mut header_data)?;
        let (rest, header) =
            crate::header::parse_header(&header_data).expect("Failed to parse haeder");

        debug_assert!(rest.is_empty());

        Ok(Self { file, header })
    }

    pub fn parse_header(&self) -> &PMTilesHeaderV3 {
        &self.header
    }

    pub fn parse_metadata(&mut self) -> Result<String, Box<dyn Error>> {
        let metadata_decoded = decompress(
            &mut self.file,
            self.header.metadata_offset,
            self.header.metadata_length as usize,
            &self.header.internal_compression,
        )?;

        Ok(String::from_utf8(metadata_decoded)?)
    }

    fn parse_directory(
        &mut self,
        offset: u64,
        length: usize,
    ) -> Result<Vec<PMTilesEntry>, Box<dyn Error>> {
        let decoded = decompress(
            &mut self.file,
            offset,
            length,
            &self.header.internal_compression,
        )?;
        let (rest, entries) =
            directory::parse_directory(&decoded).expect("Failed to parse directory");

        debug_assert!(rest.is_empty());

        Ok(entries)
    }

    pub fn parse_root_directory(&mut self) -> Result<Vec<PMTilesEntry>, Box<dyn Error>> {
        self.parse_directory(
            self.header.root_directory_offset,
            self.header.root_directory_length as usize,
        )
    }

    pub fn parse_leaf_directory(
        &mut self,
        offset: u64,
        length: usize,
    ) -> Result<Vec<PMTilesEntry>, Box<dyn Error>> {
        self.parse_directory(self.header.leaf_directories_offset + offset, length)
    }

    pub fn parse_tile(&mut self, offset: u64, length: usize) -> Result<mvt::Tile, Box<dyn Error>> {
        let tile_decoded = decompress(
            &mut self.file,
            self.header.tile_data_offset + offset,
            length,
            &self.header.tile_compression,
        )?;
        let tile = mvt::Tile::decode(tile_decoded.as_slice())?;

        Ok(tile)
    }
}

pub(crate) fn decompress(
    file: &mut std::fs::File,
    offset: u64,
    length: usize,
    compression: &PMTilesCompression,
) -> Result<Vec<u8>, Box<dyn Error>> {
    // leaf dir might not exist
    if length == 0 {
        return Ok(vec![]);
    }

    let mut raw_bytes = vec![0u8; length];
    file.seek(std::io::SeekFrom::Start(offset))?;
    file.read_exact(&mut raw_bytes)?;

    let mut decoded = vec![];
    let mut decoder = match &compression {
        PMTilesCompression::Unknown => unimplemented!(),
        PMTilesCompression::None => return Ok(raw_bytes),
        PMTilesCompression::Gzip => GzDecoder::new(raw_bytes.as_slice()),
        PMTilesCompression::Brotli => unimplemented!(),
        PMTilesCompression::Zstd => unimplemented!(),
    };
    decoder.read_to_end(&mut decoded)?;
    Ok(decoded)
}

pub(crate) fn print_tile_value(value: &mvt::tile::Value) {
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
