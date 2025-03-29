use std::{
    error::Error,
    io::{Read as _, Seek as _},
};

use flate2::read::GzDecoder;
use nom::error::ErrorKind;

use crate::{header::PMTilesCompression, mvt};

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
