use flate2::read::GzDecoder;
use mapper_abi::*;
use vastrum_rpc_client::SentTxBehavior;
use std::io::Read;
use std::mem::take;

pub const UPLOAD_CONCURRENCY: usize = 50;
pub const BATCH_SIZE_BUDGET: usize = 3_500_000;

pub fn decompress_tile(data: Vec<u8>) -> Vec<u8> {
    if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b {
        let mut decoder = GzDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap_or_default();
        return decompressed;
    }
    return data;
}

pub fn read_mbtiles_metadata(db: &rusqlite::Connection) -> MapMetadata {
    let get = |name: &str| -> String {
        db.query_row("SELECT value FROM metadata WHERE name = ?1", [name], |row| row.get(0))
            .unwrap_or_default()
    };

    let min_zoom: u8 = get("minzoom").parse().unwrap_or(0);
    let max_zoom: u8 = get("maxzoom").parse().unwrap_or(14);

    let center_str = get("center");
    let center_parts: Vec<f64> =
        center_str.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let (center_lng, center_lat) = if center_parts.len() >= 2 {
        ((center_parts[0] * 1_000_000.0) as i64, (center_parts[1] * 1_000_000.0) as i64)
    } else {
        (0, 0)
    };

    let bounds_str = get("bounds");
    let bounds_parts: Vec<f64> =
        bounds_str.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let (bounds_min_lng, bounds_min_lat, bounds_max_lng, bounds_max_lat) =
        if bounds_parts.len() >= 4 {
            (
                (bounds_parts[0] * 1_000_000.0) as i64,
                (bounds_parts[1] * 1_000_000.0) as i64,
                (bounds_parts[2] * 1_000_000.0) as i64,
                (bounds_parts[3] * 1_000_000.0) as i64,
            )
        } else {
            (-180_000_000, -85_000_000, 180_000_000, 85_000_000)
        };

    return MapMetadata {
        min_zoom,
        max_zoom,
        center_lat,
        center_lng,
        bounds_min_lat,
        bounds_min_lng,
        bounds_max_lat,
        bounds_max_lng,
    };
}

pub async fn upload_tiles(client: &ContractAbiClient, path: &str, checkpoint_path: &str) {
    println!("Opening mbtiles: {path}");
    let db = rusqlite::Connection::open(path).expect("Failed to open .mbtiles file");

    let metadata = read_mbtiles_metadata(&db);
    let max_zoom = metadata.max_zoom;
    client.set_metadata(metadata).await.await_confirmation().await;
    println!("Metadata uploaded.");

    let mut zoom_counts: Vec<(u8, u64)> = Vec::new();
    {
        let mut stmt = db
            .prepare(
                "SELECT zoom_level, COUNT(*) FROM tiles GROUP BY zoom_level ORDER BY zoom_level",
            )
            .unwrap();
        let rows =
            stmt.query_map([], |row| Ok((row.get::<_, u8>(0)?, row.get::<_, u64>(1)?))).unwrap();
        for row in rows {
            let Ok((z, count)) = row else { continue };
            zoom_counts.push((z, count));
        }
    }

    let total_tiles: u64 = zoom_counts.iter().map(|(_, c)| c).sum();
    println!(
        "Total tiles: {total_tiles} across {} zoom levels (0..={max_zoom})",
        zoom_counts.len()
    );

    let completed_zooms = read_checkpoint(checkpoint_path);
    if !completed_zooms.is_empty() {
        println!("Resuming — already completed zoom levels: {completed_zooms:?}");
    }

    let global_start = std::time::Instant::now();
    let mut global_uploaded_tiles: u64 = 0;

    for &(z, count) in &zoom_counts {
        if completed_zooms.contains(&z) {
            global_uploaded_tiles += count;
        }
    }

    for &(z, zoom_tile_count) in &zoom_counts {
        if completed_zooms.contains(&z) {
            println!("z{z}: already uploaded ({zoom_tile_count} tiles), skipping");
            continue;
        }

        println!("z{z}: uploading {zoom_tile_count} tiles...");
        let zoom_start = std::time::Instant::now();

        let mut stmt = db
            .prepare("SELECT tile_column, tile_row, tile_data FROM tiles WHERE zoom_level = ?")
            .unwrap();

        let rows = stmt
            .query_map([z], |row| {
                let x: u32 = row.get(0)?;
                let tms_y: u32 = row.get(1)?;
                let data: Vec<u8> = row.get(2)?;
                Ok((x, tms_y, data))
            })
            .unwrap();

        let mut join_set = tokio::task::JoinSet::new();
        let mut current_batch: Vec<(TileCoord, Vec<u8>)> = Vec::new();
        let mut current_batch_size: usize = 0;
        let mut zoom_uploaded: u64 = 0;
        let mut zoom_batches: u64 = 0;

        for row in rows {
            let Ok((x, tms_y, data)) = row else { continue };
            let data = decompress_tile(data);

            let y = (1u32 << z) - 1 - tms_y;
            let coord = TileCoord { z, x, y };
            let tile_size = 13 + data.len();

            if current_batch_size + tile_size > BATCH_SIZE_BUDGET && !current_batch.is_empty() {
                let batch = take(&mut current_batch);
                let batch_tiles = batch.len() as u64;
                current_batch_size = 0;

                let tx = client.upload_tiles(batch).await;
                join_set.spawn(async move { tx.await_confirmation().await });

                if join_set.len() >= UPLOAD_CONCURRENCY {
                    if let Some(res) = join_set.join_next().await {
                        if res.is_ok() {
                            zoom_batches += 1;
                        }
                    }
                }

                zoom_uploaded += batch_tiles;
                global_uploaded_tiles += batch_tiles;

                if zoom_batches % 10 == 0 {
                    let elapsed = zoom_start.elapsed().as_secs_f64();
                    let tiles_per_sec =
                        if elapsed > 0.0 { zoom_uploaded as f64 / elapsed } else { 0.0 };
                    let remaining = if tiles_per_sec > 0.0 {
                        (zoom_tile_count - zoom_uploaded) as f64 / tiles_per_sec
                    } else {
                        0.0
                    };
                    let pct = zoom_uploaded as f64 / zoom_tile_count as f64 * 100.0;
                    println!(
                        "  z{z}: {pct:.1}% | {}/{} tiles | {tiles_per_sec:.0} tiles/sec | ETA {remaining:.0}s",
                        format_count(zoom_uploaded),
                        format_count(zoom_tile_count),
                    );
                }
            }

            current_batch_size += tile_size;
            current_batch.push((coord, data));
        }

        if !current_batch.is_empty() {
            let batch = take(&mut current_batch);
            let batch_tiles = batch.len() as u64;
            let tx = client.upload_tiles(batch).await;
            join_set.spawn(async move { tx.await_confirmation().await });
            zoom_uploaded += batch_tiles;
            global_uploaded_tiles += batch_tiles;
        }

        while let Some(res) = join_set.join_next().await {
            if res.is_ok() {
                zoom_batches += 1;
            }
        }

        write_checkpoint(checkpoint_path, z);

        let zoom_elapsed = zoom_start.elapsed().as_secs_f64();
        let zoom_tps = if zoom_elapsed > 0.0 { zoom_uploaded as f64 / zoom_elapsed } else { 0.0 };
        println!(
            "z{z}: complete — {} tiles in {zoom_batches} batches ({zoom_elapsed:.1}s, {zoom_tps:.0} tiles/sec)",
            format_count(zoom_uploaded),
        );
        println!(
            "  global: {}/{} tiles ({:.1}%)",
            format_count(global_uploaded_tiles),
            format_count(total_tiles),
            global_uploaded_tiles as f64 / total_tiles as f64 * 100.0,
        );
    }

    let _ = std::fs::remove_file(checkpoint_path);

    let total_elapsed = global_start.elapsed().as_secs_f64();
    let total_tps =
        if total_elapsed > 0.0 { global_uploaded_tiles as f64 / total_elapsed } else { 0.0 };
    println!();
    println!(
        "Upload complete: {} tiles in {total_elapsed:.1}s ({total_tps:.0} tiles/sec)",
        format_count(global_uploaded_tiles),
    );
}

pub fn read_checkpoint(path: &str) -> Vec<u8> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    return content.lines().filter_map(|line| line.trim().parse::<u8>().ok()).collect();
}

pub fn write_checkpoint(path: &str, zoom: u8) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Failed to write checkpoint file");
    writeln!(file, "{zoom}").expect("Failed to write checkpoint");
}

pub fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        return format!("{:.1}M", n as f64 / 1_000_000.0);
    }
    if n >= 1_000 {
        return format!("{:.1}K", n as f64 / 1_000.0);
    }
    return format!("{n}");
}
