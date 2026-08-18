
use crate::checkpoint::{
    CHECKPOINT_EVERY_BATCHES, TilePos, append_checkpoint_line, read_checkpoint,
};
use crate::fmt::{format_bytes, format_count, format_duration};
use crate::mbtiles::{MAX_BLOB_LEN, is_open_water};
use crate::transport::{BATCH_SIZE_BUDGET, BatchPayload, UploadResult, drain, spawn_send};
use mapper_abi::*;
use std::mem::take;
use std::sync::Arc;
use vastrum_shared_types::limits::{FUEL_KV_WRITE_BASE, FUEL_KV_WRITE_PER_BYTE, TX_FUEL_CAP};

const FUEL_BUDGET_PER_TX: u64 = TX_FUEL_CAP / 4 * 3;

fn est_fuel(item_bytes: usize) -> u64 {
    (FUEL_KV_WRITE_BASE + item_bytes as u64 * FUEL_KV_WRITE_PER_BYTE) * 2
}

#[derive(Default)]
pub struct Stats {
    pub tiles_stored: u64,
    pub tile_bytes: u64,
    pub default_skipped: u64,
}

struct Batches {
    join_set: tokio::task::JoinSet<UploadResult<()>>,
    client: Arc<ContractAbiClient>,
    tiles: Vec<TileEntry>,
    tile_bytes: usize,
    tile_fuel: u64,
    since_checkpoint: u64,
}

impl Batches {
    fn new(client: Arc<ContractAbiClient>) -> Self {
        Self {
            join_set: tokio::task::JoinSet::new(),
            client,
            tiles: Vec::new(),
            tile_bytes: 0,
            tile_fuel: 0,
            since_checkpoint: 0,
        }
    }

    async fn push_tile(&mut self, coord: TileCoord, data: Vec<u8>) -> UploadResult<()> {
        let fuel = est_fuel(data.len());
    if !self.tiles.is_empty()
            && (self.tile_bytes + data.len() + 24 > BATCH_SIZE_BUDGET
                || self.tile_fuel + fuel > FUEL_BUDGET_PER_TX)
        {
            let payload = BatchPayload::Tiles(take(&mut self.tiles));
            self.tile_bytes = 0;
            self.tile_fuel = 0;
            spawn_send(&mut self.join_set, self.client.clone(), payload).await?;
            self.since_checkpoint += 1;
        }
        self.tile_bytes += data.len() + 24;
        self.tile_fuel += fuel;
        self.tiles.push(TileEntry { coord, data });
        return Ok(());
    }

    async fn flush_and_drain(&mut self) -> UploadResult<()> {
        if !self.tiles.is_empty() {
            let payload = BatchPayload::Tiles(take(&mut self.tiles));
            self.tile_bytes = 0;
            self.tile_fuel = 0;
            spawn_send(&mut self.join_set, self.client.clone(), payload).await?;
        }
        drain(&mut self.join_set).await?;
        self.since_checkpoint = 0;
        return Ok(());
    }
}

pub async fn upload_tiles(
    client: Arc<ContractAbiClient>,
    path: &str,
    checkpoint_path: &str,
) -> UploadResult<()> {
    println!("Opening mbtiles: {path}");
    let db = rusqlite::Connection::open(path).expect("Failed to open .mbtiles file");

    struct ZoomCount {
        zoom: u8,
        tiles: u64,
    }
    let mut zoom_counts: Vec<ZoomCount> = Vec::new();
    {
        let mut stmt = db
            .prepare(
                "SELECT zoom_level, COUNT(*) FROM tiles GROUP BY zoom_level ORDER BY zoom_level",
            )
            .unwrap();
        let rows =
            stmt.query_map([], |row| Ok((row.get::<_, u8>(0)?, row.get::<_, u64>(1)?))).unwrap();
        for row in rows {
            let Ok((zoom, tiles)) = row else { continue };
            zoom_counts.push(ZoomCount { zoom, tiles });
        }
    }
    let total_tiles: u64 = zoom_counts.iter().map(|zc| zc.tiles).sum();
    println!("Total tiles: {total_tiles} across {} zoom levels", zoom_counts.len());

    let checkpoint = read_checkpoint(checkpoint_path);
    if !checkpoint.completed_zooms.is_empty() || !checkpoint.positions.is_empty() {
        println!(
            "Resuming — completed zooms: {:?}, partial zooms: {:?}",
            checkpoint.completed_zooms,
            checkpoint.positions.keys().collect::<Vec<_>>()
        );
    }

    let global_start = std::time::Instant::now();
    let mut stats = Stats::default();

    for zc in &zoom_counts {
        if checkpoint.completed_zooms.contains(&zc.zoom) {
            println!("z{}: already uploaded ({} tiles), skipping", zc.zoom, zc.tiles);
            continue;
        }
        upload_zoom(
            &client,
            &db,
            zc.zoom,
            zc.tiles,
            checkpoint.positions.get(&zc.zoom).copied(),
            &mut stats,
            checkpoint_path,
        )
        .await?;
    }

    let total_elapsed = global_start.elapsed().as_secs_f64();
    println!();
    println!(
        "Upload complete in {}: {} tiles stored ({}), {} tiles default-skipped",
        format_duration(total_elapsed),
        format_count(stats.tiles_stored),
        format_bytes(stats.tile_bytes),
        format_count(stats.default_skipped),
    );
    return Ok(());
}

async fn upload_zoom(
    client: &Arc<ContractAbiClient>,
    db: &rusqlite::Connection,
    z: u8,
    zoom_tile_count: u64,
    resume: Option<TilePos>,
    stats: &mut Stats,
    checkpoint_path: &str,
) -> UploadResult<()> {
    println!("z{z}: uploading {zoom_tile_count} tiles...");
    let zoom_start = std::time::Instant::now();
    let max_y = (1u32 << z) - 1;

    // Ordered by (column, row) so a checkpointed position resumes cleanly.
    let mut stmt = db
        .prepare(
            "SELECT tile_column, tile_row, tile_data FROM tiles WHERE zoom_level = ?1 \
             AND (?2 IS NULL OR tile_column > ?2 \
                  OR (tile_column = ?2 AND tile_row > ?3)) \
             ORDER BY tile_column, tile_row",
        )
        .unwrap();
    let (resume_col, resume_row) = match resume {
        Some(p) => (Some(p.col), p.row),
        None => (None, 0),
    };
    let rows = stmt
        .query_map(rusqlite::params![z, resume_col, resume_row], |row| {
            let col: u32 = row.get(0)?;
            let tms_y: u32 = row.get(1)?;
            let data: Vec<u8> = row.get(2)?;
            Ok((col, tms_y, data))
        })
        .unwrap();

    let mut batches = Batches::new(client.clone());
    let mut zoom_processed: u64 = 0;

    for row in rows {
        let Ok((col, tms_y, data)) = row else { continue };
        zoom_processed += 1;
        let y = max_y - tms_y;
        if data.len() > MAX_BLOB_LEN {
            eprintln!("skipping oversized tile z{z} x{col} tms_y{tms_y}: {} bytes", data.len());
        } else if is_open_water(&data) {
            stats.default_skipped += 1;
        } else {
            stats.tiles_stored += 1;
            stats.tile_bytes += data.len() as u64;
            batches.push_tile(TileCoord { z, x: col, y }, data).await?;
        }

        if batches.since_checkpoint >= CHECKPOINT_EVERY_BATCHES {
            batches.flush_and_drain().await?;
            append_checkpoint_line(checkpoint_path, &format!("pos {z} {col} {tms_y}"));
            print_progress(z, zoom_processed, zoom_tile_count, &zoom_start, stats);
        }
    }

    batches.flush_and_drain().await?;
    append_checkpoint_line(checkpoint_path, &format!("done {z}"));

    let zoom_elapsed = zoom_start.elapsed().as_secs_f64();
    println!(
        "z{z}: complete — {} tiles in {} ({:.0} tiles/sec)",
        format_count(zoom_processed),
        format_duration(zoom_elapsed),
        zoom_processed as f64 / zoom_elapsed.max(0.001),
    );
    return Ok(());
}

fn print_progress(
    z: u8,
    zoom_processed: u64,
    zoom_tile_count: u64,
    zoom_start: &std::time::Instant,
    stats: &Stats,
) {
    let elapsed = zoom_start.elapsed().as_secs_f64();
    let tps = zoom_processed as f64 / elapsed.max(0.001);
    let remaining = (zoom_tile_count.saturating_sub(zoom_processed)) as f64 / tps.max(0.001);
    println!(
        "  z{z}: {}/{} tiles | {tps:.0} tiles/sec | {} stored ({}) | {} default-skipped | ETA {}",
        format_count(zoom_processed),
        format_count(zoom_tile_count),
        format_count(stats.tiles_stored),
        format_bytes(stats.tile_bytes),
        format_count(stats.default_skipped),
        format_duration(remaining),
    );
}
