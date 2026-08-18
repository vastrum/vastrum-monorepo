const WATER_TILE: &[u8] = include_bytes!("water.mvt");

#[wasm_bindgen]
pub async fn get_tile(z: u8, x: u32, y: u32) -> Vec<u8> {
    let key = TileKey { z, x, y };
    if let Some(tile) = cached_tile(key) {
        return tile;
    }
    let state = contract_state().await;
    let tile = match state.tiles.get(&TileCoord { z, x, y }).await {
        Some(data) => decompress_if_gzip(data),
        None => WATER_TILE.to_vec(),
    };
    cache_tile(key, tile.clone());
    return tile;
}

struct ClearCache<K, V> {
    map: std::collections::HashMap<K, V>,
    max: usize,
}

impl<K: std::hash::Hash + Eq, V: Clone> ClearCache<K, V> {
    fn new(max: usize) -> Self {
        Self { map: std::collections::HashMap::new(), max }
    }
    fn get<Q: std::hash::Hash + Eq + ?Sized>(&self, k: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.get(k).cloned()
    }
    fn insert(&mut self, k: K, v: V) {
        if self.map.len() >= self.max {
            self.map.clear();
        }
        self.map.insert(k, v);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TileKey {
    z: u8,
    x: u32,
    y: u32,
}

thread_local! {
    static TILE_CACHE: std::cell::RefCell<ClearCache<TileKey, Vec<u8>>> =
        std::cell::RefCell::new(ClearCache::new(512));
}

fn cached_tile(key: TileKey) -> Option<Vec<u8>> {
    TILE_CACHE.with(|c| c.borrow().get(&key))
}

fn cache_tile(key: TileKey, tile: Vec<u8>) {
    TILE_CACHE.with(|c| c.borrow_mut().insert(key, tile));
}

#[wasm_bindgen]
pub async fn search_places(query: String, center_lat: f64, center_lng: f64) -> JsSearchResults {
    use mapper_abi::search::{BUCKET_CAP, BUCKET_PREFIX_LEN};
    const MAX_RESULTS: usize = 15;

    let normalized = normalize(&query);
    let tokens: Vec<&str> =
        normalized.split(|c: char| !c.is_alphanumeric()).filter(|t| !t.is_empty()).collect();

    let mut key_tokens: Vec<&&str> =
        tokens.iter().filter(|t| t.len() >= BUCKET_PREFIX_LEN).collect();
    key_tokens.sort_by_key(|t| std::cmp::Reverse(t.len()));
    key_tokens.truncate(3);
    if key_tokens.is_empty() {
        return JsSearchResults { results: Vec::new() };
    }
    let mut bucket: Vec<PlaceEntry> = Vec::new();
    let mut seen_entries: std::collections::HashSet<mapper_abi::search::EntryKey> =
        std::collections::HashSet::new();
    for key_token in key_tokens {
        let key: String = key_token.chars().take(BUCKET_PREFIX_LEN).collect();
        let mut part = fetch_bucket(&key).await;

        if part.len() == BUCKET_CAP
            && let Some(key4) = mapper_abi::search::spill_key(key_token)
        {
            part.extend(fetch_bucket(&key4).await);
        }
        for entry in part {
            if seen_entries.insert(mapper_abi::search::entry_key(&entry)) {
                bucket.push(entry);
            }
        }
    }

    let mut scored: Vec<Scored> = Vec::new();
    for entry in &bucket {
        if let Some(score) = name_score(entry, &tokens) {
            scored.push(Scored { score, entry });
        }
    }

    if scored.len() < 5 {
        for entry in &bucket {
            if scored.iter().any(|s| std::ptr::eq(s.entry, entry)) {
                continue;
            }
            let entry_norm = normalize(&entry.name);
            let entry_tokens: Vec<&str> = entry_norm
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| !t.is_empty())
                .collect();
            let fuzzy_ok = tokens.iter().all(|qt| {
                entry_tokens.iter().any(|et| {
                    token_matches(qt, et) || within_edit_distance(qt, et, typo_budget(qt))
                })
            });
            if fuzzy_ok {
                scored.push(Scored { score: SCORE_FUZZY, entry });
            }
        }
    }

    scored.sort_by(|a, b| {
        let da = view_distance2(a.entry, center_lat, center_lng);
        let db = view_distance2(b.entry, center_lat, center_lng);
        a.score
            .cmp(&b.score)
            .then(a.entry.rank.cmp(&b.entry.rank))
            .then(b.entry.population.cmp(&a.entry.population))
            .then(da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.entry.name.len().cmp(&b.entry.name.len()))
    });
    scored.truncate(MAX_RESULTS);

    let results = scored
        .into_iter()
        .map(|s| JsPlace {
            name: s.entry.name.clone(),
            lat: s.entry.lat as f64 / 1_000_000.0,
            lng: s.entry.lng as f64 / 1_000_000.0,
            rank: s.entry.rank,
        })
        .collect();
    return JsSearchResults { results };
}

struct Scored<'a> {
    score: u8,
    entry: &'a PlaceEntry,
}

const SCORE_FUZZY: u8 = 3;

fn name_score(entry: &PlaceEntry, query_tokens: &[&str]) -> Option<u8> {
    let entry_norm = normalize(&entry.name);
    let entry_tokens: Vec<&str> =
        entry_norm.split(|c: char| !c.is_alphanumeric()).filter(|t| !t.is_empty()).collect();
    if !query_tokens.iter().all(|qt| entry_tokens.iter().any(|et| token_matches(qt, et))) {
        return None;
    }
    let all_covered =
        entry_tokens.iter().all(|et| query_tokens.iter().any(|qt| token_matches(qt, et)));
    let all_exact = query_tokens.iter().all(|qt| entry_tokens.iter().any(|et| et == qt));
    if all_exact && all_covered && entry_tokens.len() == query_tokens.len() {
        return Some(0);
    }
    if all_covered {
        return Some(1);
    }
    return Some(2);
}

fn view_distance2(entry: &PlaceEntry, center_lat: f64, center_lng: f64) -> f64 {
    let lat = entry.lat as f64 / 1_000_000.0;
    let lng = entry.lng as f64 / 1_000_000.0;
    let cos = center_lat.to_radians().cos().max(0.05);
    let dlat = lat - center_lat;
    let mut dlng = (lng - center_lng).abs() % 360.0;
    if dlng > 180.0 {
        dlng = 360.0 - dlng;
    }
    return dlat * dlat + (dlng * cos) * (dlng * cos);
}

fn typo_budget(token: &str) -> usize {
    match token.chars().count() {
        0..=3 => 0,
        4..=6 => 1,
        _ => 2,
    }
}

/// Bounded Levenshtein: true if edit distance between a and b is <= budget.
/// Tokens are short, so plain DP with early exit is plenty fast.
fn within_edit_distance(a: &str, b: &str, budget: usize) -> bool {
    if budget == 0 {
        return a == b;
    }
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > budget {
        return false;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut row = vec![i];
        let mut row_min = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            let v = (prev[j] + 1).min(row[j - 1] + 1).min(prev[j - 1] + cost);
            row_min = row_min.min(v);
            row.push(v);
        }
        if row_min > budget {
            return false;
        }
        prev = row;
    }
    return prev[b.len()] <= budget;
}

use mapper_abi::search::{normalize, token_matches};

async fn fetch_bucket(key: &str) -> Vec<PlaceEntry> {
    if let Some(bucket) = cached_bucket(key) {
        return bucket;
    }
    let state = contract_state().await;
    let bucket = state.search_buckets.get(&key.to_string()).await.unwrap_or_default();
    cache_bucket(key.to_string(), bucket.clone());
    return bucket;
}

thread_local! {
    static BUCKET_CACHE: std::cell::RefCell<ClearCache<String, Vec<PlaceEntry>>> =
        std::cell::RefCell::new(ClearCache::new(24));
}

fn cached_bucket(key: &str) -> Option<Vec<PlaceEntry>> {
    BUCKET_CACHE.with(|c| c.borrow().get(key))
}

fn cache_bucket(key: String, bucket: Vec<PlaceEntry>) {
    BUCKET_CACHE.with(|c| c.borrow_mut().insert(key, bucket));
}

#[derive(serde::Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct JsSearchResults {
    pub results: Vec<JsPlace>,
}

#[derive(serde::Serialize, Tsify)]
pub struct JsPlace {
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub rank: u8,
}

use mapper_abi::tile_block::decompress_if_gzip;

fn new_client() -> ContractAbiClient {
    return ContractAbiClient::new(Sha256Digest::from([0u8; 32]));
}

thread_local! {
    static CONTRACT_STATE: std::cell::RefCell<Option<std::rc::Rc<NativeContract>>> =
        std::cell::RefCell::new(None);
}

async fn contract_state() -> std::rc::Rc<NativeContract> {
    if let Some(state) = CONTRACT_STATE.with(|c| c.borrow().clone()) {
        return state;
    }
    let state = std::rc::Rc::new(new_client().state().await);
    CONTRACT_STATE.with(|c| *c.borrow_mut() = Some(state.clone()));
    return state;
}

pub use mapper_abi::*;
use tsify::Tsify;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use wasm_bindgen::prelude::*;
