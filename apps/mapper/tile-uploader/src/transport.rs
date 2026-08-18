use mapper_abi::*;
use std::sync::Arc;
use vastrum_rpc_client::SentTxBehavior;

#[derive(Debug)]
pub struct UploadError(pub String);

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "upload error: {}", self.0)
    }
}
impl std::error::Error for UploadError {}

pub type UploadResult<T> = Result<T, UploadError>;

pub const UPLOAD_CONCURRENCY: usize = 16;

pub const BATCH_SIZE_BUDGET: usize = 2_500_000;
const SEND_ATTEMPTS: usize = 30;
const TX_CAP: usize = 4 * 1024 * 1024;
const _: () = assert!(crate::mbtiles::MAX_BLOB_LEN <= BATCH_SIZE_BUDGET);
const _: () = assert!(BATCH_SIZE_BUDGET + 512 * 1024 <= TX_CAP);

pub(crate) enum BatchPayload {
    Tiles(Vec<TileEntry>),
    SearchBuckets(Vec<SearchBucket>),
}

pub(crate) async fn send_batch(
    client: Arc<ContractAbiClient>,
    payload: BatchPayload,
) -> UploadResult<()> {
    for attempt in 1..=SEND_ATTEMPTS {
        let tx = match &payload {
            BatchPayload::Tiles(tiles) => client.upload_tiles(tiles.clone()).await,
            BatchPayload::SearchBuckets(buckets) => {
                client.upload_search_buckets(buckets.clone()).await
            }
        };
        tx.await_confirmation().await;
        if tx.check_if_included().await {
            return Ok(());
        }

        let backoff = std::cmp::min(2u64.saturating_mul(attempt as u64), 30);
        eprintln!(
            "batch tx not included (attempt {attempt}/{SEND_ATTEMPTS}), backoff {backoff}s..."
        );
        tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
    }
    return Err(UploadError(format!("batch failed to land after {SEND_ATTEMPTS} attempts")));
}

pub(crate) async fn spawn_send(
    join_set: &mut tokio::task::JoinSet<UploadResult<()>>,
    client: Arc<ContractAbiClient>,
    payload: BatchPayload,
) -> UploadResult<()> {
    join_set.spawn(send_batch(client, payload));
    while join_set.len() >= UPLOAD_CONCURRENCY {
        if let Some(res) = join_set.join_next().await {
            res.map_err(|e| UploadError(format!("upload task panicked: {e}")))??;
        }
    }
    return Ok(());
}

pub(crate) async fn drain(
    join_set: &mut tokio::task::JoinSet<UploadResult<()>>,
) -> UploadResult<()> {
    while let Some(res) = join_set.join_next().await {
        res.map_err(|e| UploadError(format!("upload task panicked: {e}")))??;
    }
    return Ok(());
}

pub(crate) async fn spawn_send_buckets(
    join_set: &mut tokio::task::JoinSet<UploadResult<()>>,
    client: Arc<ContractAbiClient>,
    buckets: Vec<SearchBucket>,
) -> UploadResult<()> {
    return spawn_send(join_set, client, BatchPayload::SearchBuckets(buckets)).await;
}
