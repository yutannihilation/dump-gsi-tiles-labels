use std::{
    error::Error,
    io::{Read as _, Seek as _},
};

use flate2::read::GzDecoder;
use nom::error::ErrorKind;

use crate::header::PMTilesCompression;

pub(crate) fn nom_error<T>(input: &[u8]) -> Result<(&[u8], T), nom::Err<nom::error::Error<&[u8]>>> {
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        ErrorKind::Fail,
    )))
}

pub(crate) fn decompress(
    file: &mut std::fs::File,
    offset: u64,
    length: usize,
    compression: &PMTilesCompression,
) -> Result<Vec<u8>, Box<dyn Error>> {
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
