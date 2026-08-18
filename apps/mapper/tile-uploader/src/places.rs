use crate::transport::BATCH_SIZE_BUDGET;
use mapper_abi::*;
use std::collections::HashMap;
use std::sync::Arc;

pub use mapper_abi::search::{BUCKET_CAP, BUCKET_PREFIX_LEN, normalize};

fn place_rank(class: &str) -> Option<u8> {
    let rank = match class {
        "country" => 0,
        "state" | "region" => 1,
        "city" => 2,
        "town" => 3,
        "village" | "suburb" | "borough" => 4,
        "hamlet" | "neighbourhood" | "quarter" => 5,
        "locality" | "island" => 6,
        _ => return None,
    };
    return Some(rank);
}

pub fn extract_places(pbf_path: &str) -> Vec<PlaceEntry> {
    use osmpbf::ElementReader;

    let reader = ElementReader::from_path(pbf_path).expect("failed to open pbf");
    return reader.par_map_reduce(element_places, Vec::new, merge_places).expect("pbf read failed");
}

/// The place (if any) in one OSM element — the parallel map step.
fn element_places(element: osmpbf::Element) -> Vec<PlaceEntry> {
    let place = match element {
        osmpbf::Element::Node(node) => extract_node(node.tags(), node.lat(), node.lon()),
        osmpbf::Element::DenseNode(node) => extract_node(node.tags(), node.lat(), node.lon()),
        _ => None,
    };
    match place {
        Some(p) => vec![p],
        None => Vec::new(),
    }
}

/// Combine two partial results — the parallel reduce step.
fn merge_places(mut a: Vec<PlaceEntry>, mut b: Vec<PlaceEntry>) -> Vec<PlaceEntry> {
    a.append(&mut b);
    return a;
}

fn extract_node<'a>(
    tags: impl Iterator<Item = (&'a str, &'a str)>,
    lat: f64,
    lon: f64,
) -> Option<PlaceEntry> {
    let mut name: Option<&str> = None;
    let mut name_en: Option<&str> = None;
    let mut name_latin: Option<&str> = None;
    let mut class: Option<&str> = None;
    let mut population: u32 = 0;
    for (k, v) in tags {
        match k {
            "name" => name = Some(v),
            "name:en" => name_en = Some(v),
            "name:latin" => name_latin = Some(v),
            "place" => class = Some(v),
            "population" => population = v.replace([' ', ','], "").parse().unwrap_or(0),
            _ => {}
        }
    }
    let name = name_en.or(name_latin).or(name)?;
    let rank = place_rank(class?)?;
    if name.is_empty() || name.len() > 200 {
        return None;
    }
    return Some(PlaceEntry {
        name: name.to_string(),
        lat: (lat * 1_000_000.0) as i64,
        lng: (lon * 1_000_000.0) as i64,
        rank,
        population,
    });
}

pub const RANK_STREET: u8 = 7;
pub const RANK_POI: u8 = 8;

pub fn extract_from_tiles(mbtiles_path: &str, zoom: u8) -> Vec<PlaceEntry> {
    use geozero::mvt::{Message, Tile};
    use std::collections::HashSet;

    let db = rusqlite::Connection::open(mbtiles_path).expect("failed to open mbtiles");
    let mut stmt = db
        .prepare("SELECT tile_column, tile_row, tile_data FROM tiles WHERE zoom_level = ?1")
        .unwrap();
    let rows = stmt
        .query_map([zoom], |row| {
            Ok((row.get::<_, u32>(0)?, row.get::<_, u32>(1)?, row.get::<_, Vec<u8>>(2)?))
        })
        .unwrap();

    let n = 1u64 << zoom;
    #[derive(PartialEq, Eq, Hash)]
    struct SeenFeature {
        cell: u64,
        rank: u8,
        name: String,
    }
    let mut seen: HashSet<SeenFeature> = HashSet::new();
    let mut entries: Vec<PlaceEntry> = Vec::new();

    for row in rows {
        let Ok((x, tms_y, data)) = row else { continue };
        let raw = crate::decompress_if_gzip(data);
        let Ok(tile) = Tile::decode(raw.as_slice()) else { continue };

        let y = (n - 1) as u32 - tms_y;
        let lng = ((x as f64 + 0.5) / n as f64) * 360.0 - 180.0;
        let merc_y = std::f64::consts::PI * (1.0 - 2.0 * (y as f64 + 0.5) / n as f64);
        let lat = merc_y.sinh().atan().to_degrees();
        let cell = (x as u64 / 2) * n + (y as u64 / 2);

        for layer in &tile.layers {
            let rank = match layer.name.as_str() {
                "transportation_name" => RANK_STREET,

                "poi" | "park" | "water_name" | "mountain_peak" | "aerodrome_label" => RANK_POI,
                _ => continue,
            };
            for feature in &layer.features {
                let mut name: Option<&str> = None;
                for pair in feature.tags.chunks_exact(2) {
                    let key = layer.keys.get(pair[0] as usize).map(String::as_str);
                    if (key == Some("name:latin") || (key == Some("name") && name.is_none()))
                        && let Some(value) = layer.values.get(pair[1] as usize)
                        && let Some(s) = &value.string_value
                    {
                        name = Some(s);
                    }
                }
                let Some(name) = name else { continue };
                if name.is_empty() || name.len() > 120 {
                    continue;
                }
                if !seen.insert(SeenFeature { cell, rank, name: name.to_string() }) {
                    continue;
                }
                entries.push(PlaceEntry {
                    name: name.to_string(),
                    lat: (lat * 1_000_000.0) as i64,
                    lng: (lng * 1_000_000.0) as i64,
                    rank,
                    population: 0,
                });
            }
        }
    }
    return entries;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GridCell {
    lat: i64,
    lng: i64,
}

impl GridCell {
    fn of(lat: i64, lng: i64, size_microdeg: i64) -> GridCell {
        return GridCell { lat: lat / size_microdeg, lng: lng / size_microdeg };
    }
}

const LOCAL_CELL_SIZE: i64 = 100_000;
const CITY_CELL_SIZE: i64 = 250_000;

fn nearest_place(
    grid: &HashMap<GridCell, Vec<usize>>,
    places: &[PlaceEntry],
    cell: GridCell,
    lat: i64,
    lng: i64,
) -> Option<usize> {
    struct Nearest {
        dist2: i64,
        idx: usize,
    }
    let mut best: Option<Nearest> = None;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let neighbor = GridCell { lat: cell.lat + dy, lng: cell.lng + dx };
            let Some(idxs) = grid.get(&neighbor) else { continue };
            for &idx in idxs {
                let p = &places[idx];
                let dist2 = (p.lat - lat).pow(2) + (p.lng - lng).pow(2);
                if best.as_ref().is_none_or(|b| dist2 < b.dist2) {
                    best = Some(Nearest { dist2, idx });
                }
            }
        }
    }
    return best.map(|b| b.idx);
}

pub fn attach_localities(entries: &mut [PlaceEntry], places: &[PlaceEntry]) {
    let mut local_grid: HashMap<GridCell, Vec<usize>> = HashMap::new();
    let mut city_grid: HashMap<GridCell, Vec<usize>> = HashMap::new();
    for (i, p) in places.iter().enumerate() {
        if (3..=5).contains(&p.rank) {
            local_grid.entry(GridCell::of(p.lat, p.lng, LOCAL_CELL_SIZE)).or_default().push(i);
        }
        if p.rank == 2 {
            city_grid.entry(GridCell::of(p.lat, p.lng, CITY_CELL_SIZE)).or_default().push(i);
        }
    }

    for entry in entries.iter_mut() {
        if entry.rank < RANK_STREET {
            continue;
        }
        let local_cell = GridCell::of(entry.lat, entry.lng, LOCAL_CELL_SIZE);
        let city_cell = GridCell::of(entry.lat, entry.lng, CITY_CELL_SIZE);
        let local = nearest_place(&local_grid, places, local_cell, entry.lat, entry.lng)
            .map(|i| places[i].name.as_str());
        let city = nearest_place(&city_grid, places, city_cell, entry.lat, entry.lng)
            .map(|i| places[i].name.as_str());
        match (local, city) {
            (Some(l), Some(c)) if l != c => entry.name = format!("{} — {}, {}", entry.name, l, c),
            (Some(l), _) => entry.name = format!("{} — {}", entry.name, l),
            (None, Some(c)) => entry.name = format!("{} — {}", entry.name, c),
            (None, None) => {}
        }
    }
}

pub fn build_buckets(places: Vec<PlaceEntry>) -> HashMap<String, Vec<PlaceEntry>> {
    let mut buckets: HashMap<String, Vec<PlaceEntry>> = HashMap::new();
    for place in places {
        let own_name = place.name.split(" — ").next().unwrap_or(&place.name);
        let normalized = normalize(own_name);
        let mut seen_keys: Vec<String> = Vec::new();
        for token in normalized.split(|c: char| !c.is_alphanumeric()) {
            let mut forms: Vec<String> = vec![token.to_string()];
            forms.extend(mapper_abi::search::expand_token(token));
            for form in forms {
                if form.len() < BUCKET_PREFIX_LEN {
                    continue;
                }
                let key: String = form.chars().take(BUCKET_PREFIX_LEN).collect();
                if seen_keys.contains(&key) {
                    continue;
                }
                seen_keys.push(key.clone());
                buckets.entry(key).or_default().push(place.clone());
            }
        }
    }

    let mut spill: HashMap<String, Vec<PlaceEntry>> = HashMap::new();
    for (key, entries) in buckets.iter_mut() {
        entries.sort_by(rank_order);
        if entries.len() <= BUCKET_CAP {
            continue;
        }
        for entry in entries.split_off(BUCKET_CAP) {
            let normalized = normalize(&entry.name);
            for token in normalized.split(|c: char| !c.is_alphanumeric()) {
                if token.starts_with(key.as_str())
                    && let Some(key4) = mapper_abi::search::spill_key(token)
                {
                    spill.entry(key4).or_default().push(entry.clone());
                    break;
                }
            }
        }
    }
    for (key, mut entries) in spill {
        entries.sort_by(rank_order);
        entries.truncate(BUCKET_CAP);
        buckets.entry(key).or_insert(entries);
    }
    return buckets;
}

fn rank_order(a: &PlaceEntry, b: &PlaceEntry) -> std::cmp::Ordering {
    return a
        .rank
        .cmp(&b.rank)
        .then(b.population.cmp(&a.population))
        .then(a.name.len().cmp(&b.name.len()));
}

pub async fn upload_search_buckets(
    client: Arc<ContractAbiClient>,
    buckets: HashMap<String, Vec<PlaceEntry>>,
) -> crate::UploadResult<()> {
    let total_buckets = buckets.len();
    let mut batch: Vec<SearchBucket> = Vec::new();
    let mut batch_size: usize = 0;
    let mut sent_buckets = 0;
    let mut join_set = tokio::task::JoinSet::new();

    for (key, entries) in buckets {
        let entry_size: usize =
            entries.iter().map(|e| e.name.len() + 25).sum::<usize>() + key.len() + 8;
        if batch_size + entry_size > BATCH_SIZE_BUDGET && !batch.is_empty() {
            let payload = std::mem::take(&mut batch);
            sent_buckets += payload.len();
            batch_size = 0;
            crate::spawn_send_buckets(&mut join_set, client.clone(), payload).await?;
        }
        batch_size += entry_size;
        batch.push(SearchBucket { key, entries });
    }
    if !batch.is_empty() {
        sent_buckets += batch.len();
        crate::spawn_send_buckets(&mut join_set, client.clone(), batch).await?;
    }
    while let Some(res) = join_set.join_next().await {
        res.map_err(|e| crate::UploadError(format!("bucket upload task panicked: {e}")))??;
    }
    println!("search buckets uploaded: {sent_buckets}/{total_buckets}");
    return Ok(());
}

pub async fn build_and_upload_search_index(
    pbf: &str,
    mbtiles: Option<&str>,
    client: Arc<ContractAbiClient>,
) -> crate::UploadResult<()> {
    println!("extracting places from {pbf}");
    let start = std::time::Instant::now();
    let places = extract_places(pbf);
    println!("  {} places in {:.0}s", places.len(), start.elapsed().as_secs_f64());

    let mut all = places.clone();
    if let Some(mbtiles) = mbtiles {
        println!("extracting streets + POIs from {mbtiles} (z14)");
        let start = std::time::Instant::now();
        let mut from_tiles = extract_from_tiles(mbtiles, 14);
        println!(
            "  {} street/POI entries in {:.0}s",
            from_tiles.len(),
            start.elapsed().as_secs_f64()
        );
        attach_localities(&mut from_tiles, &places);
        all.append(&mut from_tiles);
    }

    let buckets = build_buckets(all);
    let total: usize = buckets.values().map(|v| v.len()).sum();
    println!("built {} buckets ({} indexed entries)", buckets.len(), total);

    return upload_search_buckets(client, buckets).await;
}
