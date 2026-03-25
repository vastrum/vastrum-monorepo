use wasm_bindgen::JsValue;

#[derive(thiserror::Error, Debug)]
pub enum WasmErr {
    #[error("browser API unavailable: {0}")]
    BrowserApi(&'static str),

    #[error("js: {0}")]
    Js(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("not connected")]
    NotConnected,

    #[error("channel closed")]
    ChannelClosed,

    #[error("request timed out")]
    RequestTimeout,

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("borsh decode: {0}")]
    BorshDecode(#[from] borsh::io::Error),

    #[error("invalid url scheme")]
    InvalidUrl,

    #[error("brotli decompress: {0}")]
    BrotliDecompress(#[from] vastrum_shared_types::compression::brotli::BrotliError),

    #[error("payload too large")]
    PayloadTooLarge,

    #[error(transparent)]
    ProofVerification(#[from] vastrum_shared_types::proof_verification::ProofVerificationError),

    #[error(transparent)]
    WebRtc(#[from] webrtc_direct_client::WebRtcError),
}

pub type Result<T> = std::result::Result<T, WasmErr>;

impl From<JsValue> for WasmErr {
    fn from(val: JsValue) -> Self {
        WasmErr::Js(format!("{:?}", val))
    }
}
impl From<WasmErr> for String {
    fn from(e: WasmErr) -> String {
        e.to_string()
    }
}
