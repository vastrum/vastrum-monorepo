use rand::Rng;
use shared_types::{
    borsh::BorshExt,
    crypto::{ed25519, sha256::Sha256Digest},
    types::{
        application::{
            sitecall::SiteCall,
            transactiondata::{TransactionData, TransactionType},
        },
        execution::transaction::Transaction,
        rpc::types::SubmitTransactionPayload,
    },
};

pub async fn make_call(site_id: Sha256Digest, calldata: String) {
    let _url = get_rpc_endpoint("submit".to_string());

    let private_key = ed25519::PrivateKey::from_seed(44);

    let call_tx_data = TransactionData {
        transaction_type: TransactionType::Call,
        calldata: SiteCall {
            site_id: site_id,
            args: calldata.as_bytes().to_vec(),
        }
        .encode(),
    };

    let nonce = rand::thread_rng().r#gen();

    let transaction = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(call_tx_data.calculate_hash()),
        calldata: call_tx_data.encode(),
        pow_nonce: nonce,
    };

    let _payload = SubmitTransactionPayload {
        transaction_bytes: transaction.encode(),
    };

    /*

    // Serialize to JSON
    let json_body = serde_json::to_string(&payload)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&json_body));

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Accept", "application/json")?;
    request.headers().set("Content-Type", "application/json")?;

    let window = web_sys::window().ok_or("no window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str(&format!("HTTP error: {}", resp.status())));
    }

    return Ok(JsValue::from_str("ok"));
     */
}
pub fn get_rpc_endpoint(route: String) -> String {
    let rpc_url = option_env!("RPC_URL").unwrap_or("http://127.0.0.1:3000/");
    let url = format!("{}{}/", rpc_url, route);
    return url;
}
