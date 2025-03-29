（開発中）
==============================

> [!WARNING]
> 特定のやりたいことのために作ったものなので、汎用的なツールにする予定はあまりないです。

## Usages

### Show header

```sh
./dump-gsi-tiles-labels show-header /path/to/optimal_bvmap-v1.pmtiles
```
```
PMTilesHeaderV3 {
    root_directory_offset: 127,
    root_directory_length: 398,
    metadata_offset: 525,
    metadata_length: 21713,
    leaf_directories_offset: 22238,
    leaf_directories_length: 0,
    tile_data_offset: 22238,
    tile_data_length: 6578918,
    number_of_addressed_tiles: 92,
    number_of_tile_entries: 92,
    number_of_tile_contents: 92,
    clustered: true,
    internal_compression: Gzip,
    tile_compression: Gzip,
    tile_type: Mvt,
    min_zoom: 0,
    max_zoom: 15,
    min_position: PMTilesPosition {
        lon: 11.221144,
        lat: 43.74512,
    },
    max_position: PMTilesPosition {
        lon: 11.287543,
        lat: 43.789307,
    },
    center_zoom: 0,
    center_position: PMTilesPosition {
        lon: 11.254343,
        lat: 43.767212,
    },
}
```

### Show metadata

```sh
./dump-gsi-tiles-labels show-metadata /path/to/optimal_bvmap-v1.pmtiles
```
```
metadata: {"vector_layers":[{"id":"boundaries", ...
```

### List entries

```sh
./dump-gsi-tiles-labels list /path/to/optimal_bvmap-v1.pmtiles
```
```
PMTilesEntry { tile_id: 278, offset: 0, length: 10981, is_tile: false }
└── PMTilesEntry { tile_id: 278, offset: 0, length: 221, is_tile: true }
└── PMTilesEntry { tile_id: 279, offset: 221, length: 20616, is_tile: true }
└── PMTilesEntry { tile_id: 284, offset: 20837, length: 6108, is_tile: true }
└── PMTilesEntry { tile_id: 290, offset: 26945, length: 23482, is_tile: true }
└── PMTilesEntry { tile_id: 291, offset: 50427, length: 104, is_tile: true }
└── PMTilesEntry { tile_id: 1115, offset: 50531, length: 206, is_tile: true }
└── PMTilesEntry { tile_id: 1116, offset: 50737, length: 72, is_tile: true }
└── PMTilesEntry { tile_id: 1117, offset: 50809, length: 198, is_tile: true }
└── PMTilesEntry { tile_id: 1118, offset: 51007, length: 1579, is_tile: true }
└── PMTilesEntry { tile_id: 1119, offset: 52586, length: 21824, is_tile: true }
    ...
PMTilesEntry { tile_id: 5654195, offset: 10981, length: 10819, is_tile: false }
└── PMTilesEntry { tile_id: 6014621, offset: 44696631, length: 2908, is_tile: true }
└── PMTilesEntry { tile_id: 6014622, offset: 44699539, length: 15935, is_tile: true }
└── PMTilesEntry { tile_id: 6014623, offset: 44715474, length: 25492, is_tile: true }
└── PMTilesEntry { tile_id: 6014624, offset: 44740966, length: 12138, is_tile: true }
└── PMTilesEntry { tile_id: 6014625, offset: 3944457, length: 71, is_tile: true }
└── PMTilesEntry { tile_id: 6014626, offset: 45942586, length: 11919, is_tile: true }
└── PMTilesEntry { tile_id: 6014627, offset: 45954505, length: 21794, is_tile: true }
└── PMTilesEntry { tile_id: 6014628, offset: 45976299, length: 15459, is_tile: true }
└── PMTilesEntry { tile_id: 6014629, offset: 45991758, length: 13818, is_tile: true }
└── PMTilesEntry { tile_id: 6014630, offset: 46005576, length: 16131, is_tile: true }
    ...
```

### Dump a tile

```sh
./dump-gsi-tiles-labels list /path/to/optimal_bvmap-v1.pmtiles 0 
```

```
---------------------------------------------------
name: AdmArea
features:
  - id: None, type: Polygon
  - id: None, type: Polygon
  - id: None, type: Polygon
  - id: None, type: Polygon
  - id: None, type: Polygon
  - id: None, type: Polygon
  - id: None, type: Polygon
  - id: None, type: Polygon
  - id: None, type: Polygon
  - id: None, type: Polygon
    ...
keys: []
values: []
---------------------------------------------------
name: AdmBdry
features:
  - id: None, type: Linestring
keys: ["vt_code"]
values: [1221, ]
---------------------------------------------------
name: Anno
features:
  - id: None, type: Point
  - id: None, type: Point
  - id: None, type: Point
  - id: None, type: Point
  - id: None, type: Point
  - id: None, type: Point
  - id: None, type: Point
  - id: None, type: Point
  - id: None, type: Point
  - id: None, type: Point
    ...
keys: ["vt_code", "vt_text"]
values: [344, "日本海", 1303, "豊岡", 1302, "姫路", "加古川", 352, "淡路島", "洲本", ...]
```

## LICENSE

- `test_fixture_1.pmtiles`: from [protomaps/go-pmtiles](https://github.com/protomaps/go-pmtiles), licensed under the BSD-3-Clause license.
- `mvt.proto`: from [mapbox/vector-tile-spec](https://github.com/mapbox/vector-tile-spec), licensed under the CC-BY-3.0 US license.
