#[contract_state]
struct Contract {
    tiles: KvMap<TileCoord, Vec<u8>>,
    search_buckets: KvMap<String, Vec<PlaceEntry>>,
    admin: Ed25519PublicKey,
}

#[contract_methods]
impl Contract {
    #[authenticated]
    pub fn upload_tiles(&mut self, tiles: Vec<TileEntry>) {
        if message_sender() != self.admin {
            return;
        }
        for tile in tiles {
            self.tiles.set(&tile.coord, tile.data);
        }
    }

    #[authenticated]
    pub fn upload_search_buckets(&mut self, buckets: Vec<SearchBucket>) {
        if message_sender() != self.admin {
            return;
        }
        for bucket in buckets {
            self.search_buckets.set(&bucket.key, bucket.entries);
        }
    }

    #[authenticated]
    pub fn set_html(&mut self, brotli_html_content: Vec<u8>) {
        if message_sender() != self.admin {
            return;
        }
        runtime::register_static_route("", &brotli_html_content);
    }

    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>, admin: Ed25519PublicKey) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        Self { tiles: KvMap::default(), search_buckets: KvMap::default(), admin }
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
struct TileEntry {
    coord: TileCoord,
    data: Vec<u8>,
}

#[contract_type]
struct SearchBucket {
    key: String,
    entries: Vec<PlaceEntry>,
}

#[contract_type]
struct PlaceEntry {
    name: String,
    lat: i64,
    lng: i64,
    rank: u8,
    population: u32,
}

use vastrum_contract_macros::{
    authenticated, constructor, contract_methods, contract_state, contract_type,
};
use vastrum_runtime_lib::{
    Ed25519PublicKey, KvMap,
    runtime::{self, message_sender},
};
