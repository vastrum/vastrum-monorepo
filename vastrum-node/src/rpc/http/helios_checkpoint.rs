pub async fn fetch_finalized_checkpoint() -> String {
    for &provider in CONSENSUS_URLS {
        let Some(latest_slot) = fetch_latest_finalized_slot(provider).await else {
            eprintln!("Consensus provider {provider} failed to return finalized slot, trying next");
            continue;
        };

        for epoch_offset in 0..10 {
            let slot = latest_slot.saturating_sub(epoch_offset * 32);
            if let Some(checkpoint) = try_bootstrap_for_slot(provider, slot).await {
                if epoch_offset > 0 {
                    eprintln!("Using checkpoint from {epoch_offset} epoch(s) back (slot {slot})");
                }
                return checkpoint;
            }
        }
        eprintln!("Provider {provider} failed after 10 epoch attempts, trying next");
    }

    panic!(
        "All consensus providers failed to fetch a valid finalized checkpoint, retry again later"
    );
}

async fn fetch_latest_finalized_slot(provider: &str) -> Option<u64> {
    let url = format!("{provider}/eth/v1/beacon/light_client/finality_update");
    let resp = reqwest::get(&url).await.ok()?;
    let update: FinalityUpdate = resp.json().await.ok()?;
    let slot: u64 = update.data.finalized_header.beacon.slot.parse().ok()?;
    Some(slot)
}

async fn try_bootstrap_for_slot(provider: &str, slot: u64) -> Option<String> {
    let url = format!("{provider}/eth/v1/beacon/headers/{slot}");
    let resp = reqwest::get(&url).await.ok()?;
    let header: BeaconHeaderResponse = resp.json().await.ok()?;
    let checkpoint = header.data.root;

    let url = format!("{provider}/eth/v1/beacon/light_client/bootstrap/{checkpoint}");
    let resp = reqwest::get(&url).await.ok()?;
    if resp.status().is_success() { Some(checkpoint) } else { None }
}

use super::super::handlers::CONSENSUS_URLS;
use super::beacon_types::{BeaconHeaderResponse, FinalityUpdate};
