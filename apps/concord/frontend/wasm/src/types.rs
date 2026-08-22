use serde::Serialize;
use vastrum_ts_macros::TsType;

#[derive(Serialize, TsType)]
#[ts(into_wasm_abi)]
pub struct JSMessage {
    pub id: u64,
    pub content: String,
    pub author: String,
    pub timestamp: u64,
}

#[derive(Serialize, TsType)]
#[ts(into_wasm_abi)]
pub struct JSMember {
    pub pubkey: String,
    pub display_name: String,
}

#[derive(Serialize, TsType)]
#[ts(into_wasm_abi)]
pub struct JSChannel {
    pub id: u64,
    pub name: String,
    pub message_count: u64,
    pub next_message_id: u64,
}

#[derive(Serialize, TsType)]
#[ts(into_wasm_abi)]
pub struct JSServerSummary {
    pub id: u64,
    pub name: String,
    pub owner: String,
    pub member_count: u64,
}

#[derive(Serialize, TsType)]
#[ts(into_wasm_abi)]
pub struct JSServerDetail {
    pub id: u64,
    pub name: String,
    pub owner: String,
    pub members: Vec<JSMember>,
    pub channels: Vec<JSChannel>,
}

#[derive(Serialize, TsType)]
#[ts(into_wasm_abi)]
pub struct JSUserProfile {
    pub display_name: String,
    pub server_ids: Vec<u64>,
    pub dm_keys: Vec<String>,
}

#[derive(Serialize, TsType)]
#[ts(into_wasm_abi)]
pub struct JSDmSummary {
    pub other_user: String,
    pub last_message: Option<String>,
    pub last_timestamp: Option<u64>,
    pub next_message_id: u64,
}
