#[derive(Clone)]
pub struct NativeHttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl NativeHttpClient {
    pub fn new() -> Self {
        let base_url = if let Ok(url) = std::env::var("RPC_URL_HACK_FOR_GIT_RELAY") {
            url
        } else if std::env::var("VASTRUM_LOCALNET").is_ok() {
            format!("http://127.0.0.1:{HTTP_RPC_PORT}")
        } else {
            "https://rpc.vastrum.org".to_string()
        };
        Self { base_url, client: reqwest::Client::new() }
    }

    pub async fn get_latest_block_height(&self) -> Result<u64, HttpError> {
        let url = format!("{}/getlatestblockheight/", self.base_url);

        Ok(self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<GetLatestBlockHeightResponse>()
            .await?
            .height)
    }

    pub async fn submit_transaction(&self, tx_bytes: Vec<u8>) -> Result<(), HttpError> {
        let payload = SubmitTransactionPayload { transaction_bytes: tx_bytes };
        let url = format!("{}/submit/", self.base_url);
        let json_size = serde_json::to_vec(&payload).expect("serialize payload").len();
        assert!(json_size <= vastrum_shared_types::limits::MAX_RPC_BODY_SIZE, "rpc body too large");
        self.client.post(&url).json(&payload).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn get_site_id_is_deployed(&self, site_id: Sha256Digest) -> Result<bool, HttpError> {
        let payload = GetSiteIDIsDeployed { site_id };
        let url = format!("{}/getsiteidisdeployed/", self.base_url);

        Ok(self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json::<GetSiteIDIsDeployedResponse>()
            .await?
            .result)
    }

    pub async fn get_tx_hash_inclusion_state(
        &self,
        tx_hash: Sha256Digest,
    ) -> Result<bool, HttpError> {
        let payload = GetTxHashIsIncluded { tx_hash };
        let url = format!("{}/gettxhashinclusionstate/", self.base_url);

        Ok(self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json::<GetTxHashIsIncludedResponse>()
            .await?
            .included)
    }

    pub async fn get_page(
        &self,
        site_identifier: String,
        page_path: String,
    ) -> Result<GetPageResult, HttpError> {
        let payload = GetPagePayload { site_identifier, page_path };
        let url = format!("{}/page/", self.base_url);

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json::<GetPageResult>()
            .await?;
        Ok(resp)
    }

    //todo add state verification here
    //currently only used for tests so not really that important
    pub async fn get_page_content(
        &self,
        site_identifier: String,
        page_path: String,
    ) -> Result<String, HttpError> {
        let result = self.get_page(site_identifier, page_path.clone()).await?;
        match result {
            GetPageResult::Ok(resp) => {
                Ok(vastrum_shared_types::compression::brotli::brotli_decompress_html(
                    &resp.brotli_html_content,
                )?)
            }
            GetPageResult::Err(e) => Err(HttpError(format!("{e:?}"))),
        }
    }

    pub async fn wait_for_next_block(&self) {
        let height = self.get_latest_block_height().await.unwrap_or(0);
        loop {
            if self.get_latest_block_height().await.unwrap_or(0) > height {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }

    pub async fn resolve_domain(&self, domain: String) -> Result<Option<Sha256Digest>, HttpError> {
        let payload = ResolveDomainRequest { domain };
        let url = format!("{}/resolvedomain/", self.base_url);

        Ok(self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json::<ResolveDomainResponse>()
            .await?
            .site_id)
    }

    pub async fn get_key_value_response(
        &self,
        site_id: Sha256Digest,
        key: String,
        height: Option<u64>,
    ) -> Result<GetKeyValueResult, HttpError> {
        let payload = GetKeyValuePayload { key, site_id, height_lock: height };
        let url = format!("{}/getkeyvalue/", self.base_url);
        Ok(self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json::<GetKeyValueResult>()
            .await?)
    }

    pub async fn get_key_value(&self, site_id: Sha256Digest, key: String) -> Option<Vec<u8>> {
        self.get_key_value_with_height(site_id, key, None).await
    }

    pub async fn get_key_value_at_height(
        &self,
        site_id: Sha256Digest,
        key: String,
        height: u64,
    ) -> Option<Vec<u8>> {
        self.get_key_value_with_height(site_id, key, Some(height)).await
    }

    async fn get_key_value_with_height(
        &self,
        site_id: Sha256Digest,
        key: String,
        height: Option<u64>,
    ) -> Option<Vec<u8>> {
        let payload = GetKeyValuePayload { key, site_id, height_lock: height };
        let url = format!("{}/getkeyvalue/", self.base_url);

        let result = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?
            .json::<GetKeyValueResult>()
            .await
            .ok()?;
        match result {
            GetKeyValueResult::Ok(r) => Some(r.value),
            GetKeyValueResult::Err(_) => None,
        }
    }
}

impl Default for NativeHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

use crate::error::HttpError;
use vastrum_shared_types::{
    crypto::sha256::Sha256Digest,
    ports::HTTP_RPC_PORT,
    types::rpc::types::{
        GetKeyValuePayload, GetKeyValueResult, GetLatestBlockHeightResponse, GetPagePayload,
        GetPageResult, GetSiteIDIsDeployed, GetSiteIDIsDeployedResponse, GetTxHashIsIncluded,
        GetTxHashIsIncludedResponse, ResolveDomainRequest, ResolveDomainResponse,
        SubmitTransactionPayload,
    },
};
