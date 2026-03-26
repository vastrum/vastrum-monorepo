#[contract_state]
struct Contract {
    tiles: KvMap<TileCoord, Vec<u8>>,
    metadata: MapMetadata,
    admin: Ed25519PublicKey,
}

#[contract_methods]
impl Contract {
    #[authenticated]
    pub fn upload_tile(&mut self, coord: TileCoord, data: Vec<u8>) {
        if message_sender() != self.admin {
            return;
        }
        self.tiles.set(&coord, data);
    }

    #[authenticated]
    pub fn upload_tiles(&mut self, tiles: Vec<(TileCoord, Vec<u8>)>) {
        if message_sender() != self.admin {
            return;
        }
        for (coord, data) in tiles {
            self.tiles.set(&coord, data);
        }
    }

    #[authenticated]
    pub fn set_metadata(&mut self, metadata: MapMetadata) {
        if message_sender() != self.admin {
            return;
        }
        self.metadata = metadata;
    }

    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>, admin: Ed25519PublicKey) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        Self { tiles: KvMap::default(), metadata: MapMetadata::default(), admin }
    }
}

#[derive(PartialEq)]
#[contract_type]
struct TileCoord {
    z: u8,
    x: u32,
    y: u32,
}

#[contract_type]
struct MapMetadata {
    min_zoom: u8,
    max_zoom: u8,
    center_lat: i64,
    center_lng: i64,
    bounds_min_lat: i64,
    bounds_min_lng: i64,
    bounds_max_lat: i64,
    bounds_max_lng: i64,
}

use vastrum_contract_macros::{authenticated, constructor, contract_methods, contract_state, contract_type};
use vastrum_runtime_lib::{Ed25519PublicKey, KvMap, runtime::message_sender};
