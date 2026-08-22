#[cfg(not(madsim))]
pub use vastrum_asset_embed::EmbeddedAsset;

#[cfg(not(madsim))]
include!(concat!(env!("OUT_DIR"), "/web_client_assets.rs"));

#[cfg(not(madsim))]
pub fn embedded_asset(path: &str) -> Option<&'static EmbeddedAsset> {
    WEB_CLIENT_ASSETS.iter().find(|asset| asset.path == path)
}
