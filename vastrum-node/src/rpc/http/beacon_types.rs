use serde::Deserialize;

#[derive(Deserialize)]
pub struct FinalityUpdate {
    pub data: FinalityUpdateData,
}

#[derive(Deserialize)]
pub struct FinalityUpdateData {
    pub finalized_header: FinalizedHeader,
}

#[derive(Deserialize)]
pub struct FinalizedHeader {
    pub beacon: BeaconHeaderMessage,
}

#[derive(Deserialize)]
pub struct BeaconHeaderMessage {
    pub slot: String,
}

#[derive(Deserialize)]
pub struct BeaconHeaderResponse {
    pub data: BeaconHeaderData,
}

#[derive(Deserialize)]
pub struct BeaconHeaderData {
    pub root: String,
}
