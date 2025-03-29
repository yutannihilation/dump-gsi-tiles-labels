use nom::{IResult, Parser, multi::count};

use crate::varint::parse_varint;

#[derive(Debug, PartialEq)]
pub struct PMTilesEntry {
    pub tile_id: u64,
    pub offset: u64,
    pub length: u64,
    pub is_tile: bool,
}

pub(crate) fn parse_directory(input: &[u8]) -> IResult<&[u8], Vec<PMTilesEntry>> {
    let (input, entry_count) = parse_varint(input)?;
    let entry_count = entry_count as usize;
    let (input, tile_ids) = count(parse_varint, entry_count).parse(input)?;
    let (input, run_lengths) = count(parse_varint, entry_count).parse(input)?;
    let (input, lengths) = count(parse_varint, entry_count).parse(input)?;
    let (input, offsets) = count(parse_varint, entry_count).parse(input)?;

    let mut result = Vec::with_capacity(entry_count);
    let mut last_tile_id = 0;
    let mut last_offset = 0;

    for i in 0..entry_count {
        last_tile_id += tile_ids[i];
        let run_length = run_lengths[i];
        let length = lengths[i];
        last_offset = if offsets[i] == 0 && i > 0 {
            last_offset + lengths[i - 1]
        } else {
            offsets[i] - 1
        };

        if run_length == 0 {
            result.push(PMTilesEntry {
                tile_id: last_tile_id,
                offset: last_offset,
                length,
                is_tile: false,
            });
        } else {
            for j in 0..run_length {
                result.push(PMTilesEntry {
                    tile_id: last_tile_id + j,
                    offset: last_offset,
                    length,
                    is_tile: true,
                });
            }
        }
    }

    Ok((input, result))
}

#[cfg(test)]
mod tests {
    use flate2::read::GzDecoder;
    use std::io::Read;

    use super::*;

    #[test]
    fn test_parse_root_directory() {
        let data = include_bytes!("./test/test_fixture_1.pmtiles");

        let mut root_dir_decoded = vec![];
        let mut decoded_reader = GzDecoder::new(&data[127..152]);
        decoded_reader.read_to_end(&mut root_dir_decoded).unwrap();

        let (remaining, result) = parse_directory(&root_dir_decoded).expect("Failed to parse");
        assert!(remaining.is_empty());
        assert_eq!(result.len(), 1);

        assert_eq!(result[0].tile_id, 0);
        assert_eq!(result[0].offset, 0);
        assert_eq!(result[0].length, 69);
    }
}
