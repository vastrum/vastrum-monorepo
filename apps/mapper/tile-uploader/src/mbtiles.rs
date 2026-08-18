// Client-side sanity skip: a single MVT tile this large is anomalous/corrupt
// (real gzipped tiles are well under 1MB). Independent of any contract limit
// — the contract stores whatever fits in a transaction (MAX_TRANSACTION_SIZE).
pub const MAX_BLOB_LEN: usize = 2 * 1024 * 1024;

pub fn is_open_water(data: &[u8]) -> bool {
    use geozero::mvt::{Message, Tile};
    let raw = crate::decompress_if_gzip(data.to_vec());
    let Ok(tile) = Tile::decode(raw.as_slice()) else { return false };

    let [layer] = tile.layers.as_slice() else { return false };
    if layer.name != "water" || layer.features.is_empty() {
        return false;
    }
    let extent = layer.extent.unwrap_or(4096) as i64;
    let edge = extent / 32;

    let mut min_x = i64::MAX;
    let mut min_y = i64::MAX;
    let mut max_x = i64::MIN;
    let mut max_y = i64::MIN;
    for feature in &layer.features {
        let Some(bbox) = geometry_bbox(&feature.geometry) else { return false };
        min_x = min_x.min(bbox.min_x);
        min_y = min_y.min(bbox.min_y);
        max_x = max_x.max(bbox.max_x);
        max_y = max_y.max(bbox.max_y);
    }
    return min_x <= edge && min_y <= edge && max_x >= extent - edge && max_y >= extent - edge;
}

struct Bbox {
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
}

fn geometry_bbox(geometry: &[u32]) -> Option<Bbox> {
    let mut x: i64 = 0;
    let mut y: i64 = 0;
    let mut bbox = Bbox { min_x: i64::MAX, min_y: i64::MAX, max_x: i64::MIN, max_y: i64::MIN };
    let mut saw_point = false;
    let mut i = 0;
    while i < geometry.len() {
        let id = geometry[i] & 0x7;
        let count = (geometry[i] >> 3) as usize;
        i += 1;
        match id {
            1 | 2 => {
                for _ in 0..count {
                    if i + 1 >= geometry.len() {
                        return None;
                    }
                    x += zigzag(geometry[i]);
                    y += zigzag(geometry[i + 1]);
                    i += 2;
                    bbox.min_x = bbox.min_x.min(x);
                    bbox.min_y = bbox.min_y.min(y);
                    bbox.max_x = bbox.max_x.max(x);
                    bbox.max_y = bbox.max_y.max(y);
                    saw_point = true;
                }
            }
            7 => {}
            _ => return None,
        }
    }
    return if saw_point { Some(bbox) } else { None };
}

fn zigzag(n: u32) -> i64 {
    return ((n >> 1) as i64) ^ -((n & 1) as i64);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zz(n: i64) -> u32 {
        return ((n << 1) ^ (n >> 63)) as u32;
    }

    fn rect_geometry(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<u32> {
        return vec![
            (1 << 3) | 1, // MoveTo, 1 point
            zz(x0),
            zz(y0),       // -> (x0,y0)
            (3 << 3) | 2, // LineTo, 3 points
            zz(x1 - x0),
            zz(0), // -> (x1,y0)
            zz(0),
            zz(y1 - y0), // -> (x1,y1)
            zz(x0 - x1),
            zz(0),        // -> (x0,y1)
            (1 << 3) | 7, // ClosePath, 1
        ];
    }

    fn mvt_with_layers(names: &[&str]) -> Vec<u8> {
        use geozero::mvt::{Message, Tile, tile::Layer};
        let tile = Tile {
            layers: names
                .iter()
                .map(|n| Layer { name: n.to_string(), version: 2, ..Default::default() })
                .collect(),
        };
        return tile.encode_to_vec();
    }

    fn mvt_water(geometry: Vec<u32>) -> Vec<u8> {
        return mvt_water_features(vec![geometry]);
    }

    fn mvt_water_features(geometries: Vec<Vec<u32>>) -> Vec<u8> {
        use geozero::mvt::{Message, Tile, tile::Feature, tile::Layer};
        let tile = Tile {
            layers: vec![Layer {
                name: "water".to_string(),
                version: 2,
                extent: Some(4096),
                features: geometries
                    .into_iter()
                    .map(|geometry| Feature { geometry, r#type: Some(3), ..Default::default() })
                    .collect(),
                ..Default::default()
            }],
        };
        return tile.encode_to_vec();
    }

    #[test]
    fn full_extent_water_only_is_open_water() {
        // open ocean: single water layer, polygon fills the whole tile
        let tile = mvt_water(rect_geometry(0, 0, 4096, 4096));
        assert!(is_open_water(&tile));
    }

    #[test]
    fn tiling_water_rectangles_are_open_water() {
        let tile = mvt_water_features(vec![
            rect_geometry(0, 0, 2048, 2048),
            rect_geometry(2048, 0, 4096, 2048),
            rect_geometry(0, 2048, 2048, 4096),
            rect_geometry(2048, 2048, 4096, 4096),
        ]);
        assert!(is_open_water(&tile));
    }

    #[test]
    fn partial_water_only_is_not_open_water() {
        let tile = mvt_water(rect_geometry(0, 0, 2000, 4096));
        assert!(!is_open_water(&tile));
    }

    #[test]
    fn small_water_patch_is_not_open_water() {
        let tile = mvt_water(rect_geometry(1000, 1000, 2000, 2000));
        assert!(!is_open_water(&tile));
    }

    #[test]
    fn coastal_tile_with_land_layer_is_not_open_water() {
        use geozero::mvt::{Message, Tile, tile::Feature, tile::Layer};
        let tile = Tile {
            layers: vec![
                Layer {
                    name: "water".to_string(),
                    version: 2,
                    extent: Some(4096),
                    features: vec![Feature {
                        geometry: rect_geometry(0, 0, 4096, 4096),
                        r#type: Some(3),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Layer { name: "landcover".to_string(), version: 2, ..Default::default() },
            ],
        };
        assert!(!is_open_water(&tile.encode_to_vec()));
    }

    #[test]
    fn featureless_and_empty_tiles_are_not_open_water() {
        assert!(!is_open_water(&mvt_with_layers(&["landcover"]))); // land
        assert!(!is_open_water(&mvt_with_layers(&["water"]))); // water layer, no features
        assert!(!is_open_water(&mvt_with_layers(&[]))); // empty tile
    }
}

#[cfg(test)]
mod real_tile_test {
    #[test]
    fn real_z14_mid_pacific_ocean_tile_classifies_as_open_water() {
        let bytes: [u8; 131] = [
            26, 128, 1, 120, 2, 10, 5, 119, 97, 116, 101, 114, 40, 128, 32, 18, 23, 24, 3, 34, 15,
            9, 41, 168, 64, 26, 0, 209, 64, 142, 23, 0, 0, 210, 64, 15, 18, 2, 0, 0, 18, 23, 24, 3,
            34, 15, 9, 208, 64, 81, 26, 0, 140, 2, 223, 44, 0, 0, 139, 2, 15, 18, 2, 0, 0, 18, 23,
            24, 3, 34, 15, 9, 168, 64, 41, 26, 0, 210, 64, 183, 44, 0, 0, 209, 64, 15, 18, 2, 0, 0,
            18, 23, 24, 3, 34, 15, 9, 228, 22, 81, 26, 0, 140, 2, 181, 87, 0, 0, 139, 2, 15, 18, 2,
            0, 0, 26, 5, 99, 108, 97, 115, 115, 34, 7, 10, 5, 111, 99, 101, 97, 110,
        ];
        assert!(super::is_open_water(&bytes), "real deep-ocean z14 tile must be open water");
    }
}
