use crate::util;

use nom::IResult;
use nom::bytes::complete::tag;
use nom::number::complete::le_i32;
use nom::number::complete::le_u8;
use nom::number::complete::le_u64;

pub(crate) const HEADER_BYTES: usize = 127;

/// PMTiles V3 Header Data
///
/// <https://github.com/protomaps/PMTiles/blob/main/spec/v3/spec.md>
#[derive(Debug, PartialEq)]
pub struct PMTilesHeaderV3 {
    pub root_directory_offset: u64,
    pub root_directory_length: u64,
    pub metadata_offset: u64,
    pub metadata_length: u64,
    pub leaf_directories_offset: u64,
    pub leaf_directories_length: u64,
    pub tile_data_offset: u64,
    pub tile_data_length: u64,
    pub number_of_addressed_tiles: u64,
    pub number_of_tile_entries: u64,
    pub number_of_tile_contents: u64,
    pub clustered: bool,
    pub internal_compression: PMTilesCompression,
    pub tile_compression: PMTilesCompression,
    pub tile_type: PMTilesTileType,
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub min_position: PMTilesPosition,
    pub max_position: PMTilesPosition,
    pub center_zoom: u8,
    pub center_position: PMTilesPosition,
}

#[derive(Debug, PartialEq)]
pub enum PMTilesCompression {
    Unknown,
    None,
    Gzip,
    Brotli,
    Zstd,
}

#[derive(Debug, PartialEq)]
pub enum PMTilesTileType {
    Other,
    Mvt,
    Png,
    Jpeg,
    Webp,
    Avif,
}

#[derive(Debug, PartialEq)]
pub struct PMTilesPosition {
    pub(crate) lon: f32,
    pub(crate) lat: f32,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], PMTilesHeaderV3> {
    let (input, _) = tag("PMTiles")(input)?; // magic number
    let (input, _) = tag([3u8].as_slice())(input)?; // version number
    let (input, root_directory_offset) = le_u64(input)?;
    let (input, root_directory_length) = le_u64(input)?;
    let (input, metadata_offset) = le_u64(input)?;
    let (input, metadata_length) = le_u64(input)?;
    let (input, leaf_directories_offset) = le_u64(input)?;
    let (input, leaf_directories_length) = le_u64(input)?;
    let (input, tile_data_offset) = le_u64(input)?;
    let (input, tile_data_length) = le_u64(input)?;
    let (input, number_of_addressed_tiles) = le_u64(input)?;
    let (input, number_of_tile_entries) = le_u64(input)?;
    let (input, number_of_tile_contents) = le_u64(input)?;
    let (input, clustered) = parse_clustered(input)?;
    let (input, internal_compression) = parse_compression(input)?;
    let (input, tile_compression) = parse_compression(input)?;
    let (input, tile_type) = parse_tile_type(input)?;
    let (input, min_zoom) = le_u8(input)?;
    let (input, max_zoom) = le_u8(input)?;
    let (input, min_position) = parse_position(input)?;
    let (input, max_position) = parse_position(input)?;
    let (input, center_zoom) = le_u8(input)?;
    let (input, center_position) = parse_position(input)?;

    let header = PMTilesHeaderV3 {
        root_directory_offset,
        root_directory_length,
        metadata_offset,
        metadata_length,
        leaf_directories_offset,
        leaf_directories_length,
        tile_data_offset,
        tile_data_length,
        number_of_addressed_tiles,
        number_of_tile_entries,
        number_of_tile_contents,
        clustered,
        internal_compression,
        tile_compression,
        tile_type,
        min_zoom,
        max_zoom,
        min_position,
        max_position,
        center_zoom,
        center_position,
    };

    Ok((input, header))
}

pub(crate) fn parse_clustered(input: &[u8]) -> IResult<&[u8], bool> {
    let (input, clustered_raw) = le_u8(input)?;
    let clustered = match clustered_raw {
        0 => false,
        1 => true,
        _ => return util::nom_error(input),
    };
    Ok((input, clustered))
}

pub(crate) fn parse_compression(input: &[u8]) -> IResult<&[u8], PMTilesCompression> {
    let (input, compression_raw) = le_u8(input)?;
    let compression = match compression_raw {
        0 => PMTilesCompression::Unknown,
        1 => PMTilesCompression::None,
        2 => PMTilesCompression::Gzip,
        3 => PMTilesCompression::Brotli,
        4 => PMTilesCompression::Zstd,
        _ => return util::nom_error(input),
    };
    Ok((input, compression))
}

pub(crate) fn parse_tile_type(input: &[u8]) -> IResult<&[u8], PMTilesTileType> {
    let (input, tile_type_raw) = le_u8(input)?;
    let tile_type = match tile_type_raw {
        0 => PMTilesTileType::Other,
        1 => PMTilesTileType::Mvt,
        2 => PMTilesTileType::Png,
        3 => PMTilesTileType::Jpeg,
        4 => PMTilesTileType::Webp,
        5 => PMTilesTileType::Avif,
        _ => return util::nom_error(input),
    };
    Ok((input, tile_type))
}

pub(crate) fn parse_position(input: &[u8]) -> IResult<&[u8], PMTilesPosition> {
    let (input, lon) = le_i32(input)?;
    let (input, lat) = le_i32(input)?;
    let position = PMTilesPosition {
        lon: lon as f32 / 10_000_000.0,
        lat: lat as f32 / 10_000_000.0,
    };
    Ok((input, position))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let data = include_bytes!("./test/test_fixture_1.pmtiles");
        let (remaining, result) = parse_header(&data[..127]).expect("Failed to parse");
        assert!(remaining.is_empty());
        assert_eq!(result.root_directory_offset, 127);
        assert_eq!(result.root_directory_length, 25);
        assert_eq!(result.metadata_offset, 152);
        assert_eq!(result.metadata_length, 247);
        assert_eq!(result.leaf_directories_offset, 0);
        assert_eq!(result.leaf_directories_length, 0);
        assert_eq!(result.tile_data_offset, 399);
        assert_eq!(result.tile_data_length, 69);
        assert_eq!(result.number_of_addressed_tiles, 1);
        assert_eq!(result.number_of_tile_entries, 1);
        assert_eq!(result.number_of_tile_contents, 1);
        assert_eq!(result.clustered, false);
        assert_eq!(result.internal_compression, PMTilesCompression::Gzip);
        assert_eq!(result.tile_compression, PMTilesCompression::Gzip);
        assert_eq!(result.tile_type, PMTilesTileType::Mvt);
        assert_eq!(result.min_zoom, 0);
        assert_eq!(result.max_zoom, 0);
        assert_eq!(result.min_position.lon, 0.0);
        assert_eq!(result.min_position.lat, 0.0);
        assert_eq!(result.max_position.lon, 0.9999999);
        assert_eq!(result.max_position.lat, 1.0);
        assert_eq!(result.center_zoom, 0);
        assert_eq!(result.center_position.lon, 0.0);
        assert_eq!(result.center_position.lat, 0.0);
    }
}
