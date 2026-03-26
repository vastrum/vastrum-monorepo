use vastrum_git_lib::ContractAbiClient;
use vastrum_shared_types::crypto::sha256::Sha256Digest;

pub fn new_client() -> ContractAbiClient {
    ContractAbiClient::new(Sha256Digest::from([0u8; 32]))
}
