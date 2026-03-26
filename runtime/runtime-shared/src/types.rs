use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Ed25519PublicKey {
    pub bytes: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct Ed25519Signature {
    pub bytes: [u8; 64],
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct GetMessageSenderResponse {
    pub sender: Ed25519PublicKey,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct RegisterStaticRouteCall {
    pub route: String,
    pub brotli_html_content: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct KeyValueInsertCall {
    pub key: String,
    pub value: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct KeyValueReadCall {
    pub key: String,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct KeyValueReadResponse {
    pub value: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct LogCall {
    pub message: String,
}
