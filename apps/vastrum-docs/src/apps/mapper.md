# Mapper

[Mapper](https://lzdtxcpp6ivwje55o74dugj7f4vie6qzrsp6kybqyi7ofo3yt75q.vastrum.net)

Mapper is based on the OpenStreetMaps data. The map data is split into tiles with (x,y) positions + (z) zoom level.

This fits very well into the keyvalue database structure of Vastrum.

You just take every tile for every key (x,y) position and (z) zoom level and write it into a KvMap.

The frontend then queries the smart contract for different positions and zoom levels to build the map view.



```rust
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
    x: u8,
    y: u32,
    z: u32,
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
```

For the frontend map UI it is based on the mapbox-gl.

Frontend WASM reading a tile from the smart contract.
```rust
#[wasm_bindgen]
pub async fn get_tile(x: u8, y: u32, z: u32) -> Vec<u8> {
    let client = new_client();
    let state = client.state().await;
    let coord = TileCoord { x, y, z };
    return state.tiles.get(&coord).await.unwrap_or_default();
}
```

[Mapper on Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/apps/mapper)