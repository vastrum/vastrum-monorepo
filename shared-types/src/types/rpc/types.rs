#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitTransactionPayload {
    pub transaction_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPagePayload {
    pub page_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageResponse {
    pub content: String,
    pub site_id: String,
}

use serde::{Deserialize, Serialize};
