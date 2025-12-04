pub struct Execution {
    pub seen_pow_hash: HashSet<Sha256Digest>,
    application: Application,
}
impl Execution {
    pub fn execute_block(&mut self, block: NotarizedBlock) {
        for transaction in block.transactions {
            self.execute_transaction(transaction);
        }
    }
    fn execute_transaction(&mut self, transaction: Transaction) {
        let valid_transaction = self.verify_transaction(&transaction);
        if !valid_transaction {
            panic!("transaction was invalid");
            //return;
        }

        //mark pow hash as spent
        self.seen_pow_hash.insert(transaction.calculate_pow_hash());

        let txhash = transaction.calculate_txhash();
        self.application.execute_transaction(transaction.calldata, txhash);
    }
    pub fn verify_transaction(&self, transaction: &Transaction) -> bool {
        let calldata_hash = transaction.calculate_calldata_hash();
        let valid_signature =
            transaction.pub_key.verify_signature_hash(calldata_hash, &transaction.signature);

        let pow_hash = transaction.calculate_pow_hash();
        let passes_pow_threshold = pow_hash < self.pow_threshold();

        let pow_hash_not_yet_used = self.seen_pow_hash.get(&pow_hash).is_none();

        println!(
            "passes_pow_threshold:{passes_pow_threshold} pow_hash_not_yet_used:{pow_hash_not_yet_used} valid_signature:{valid_signature}"
        );
        let valid_tx = passes_pow_threshold && pow_hash_not_yet_used && valid_signature;

        return valid_tx;
    }

    fn pow_threshold(&self) -> Sha256Digest {
        return Sha256Digest::from([
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        ]);
    }

    pub fn new() -> Execution {
        return Execution { seen_pow_hash: HashSet::new(), application: Application::new() };
    }
}

use crate::{application::application::Application, consensus::types::NotarizedBlock};
use shared_types::{crypto::sha256::Sha256Digest, types::execution::transaction::Transaction};
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use shared_types::{
        borsh::BorshExt,
        crypto::{ed25519, sha256::Sha256Digest},
        types::{
            application::{
                sitecall::SiteCall,
                transactiondata::{TransactionData, TransactionType},
            },
            execution::transaction::Transaction,
        },
    };

    use crate::execution::execution::Execution;

    #[test]
    fn verify_tx() {
        unsafe {
            std::env::set_var("NODE_ID", "9999");
        }
        let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);

        let component_data = vec![123, 33, 12, 55, 123];

        let deploy_website_tx_data = TransactionData {
            transaction_type: TransactionType::DeployNewComponent,
            calldata: component_data,
        };
        let calldata_hash = deploy_website_tx_data.calculate_hash();

        let deploy_website_tx = Transaction {
            pub_key: private_key.public_key(),
            signature: private_key.sign_hash(calldata_hash),
            calldata: deploy_website_tx_data.encode(),
            pow_nonce: 0,
        };

        let execution = Execution::new();

        assert!(execution.verify_transaction(&deploy_website_tx));
    }

    #[test]
    fn verify_call_tx() {
        unsafe {
            std::env::set_var("NODE_ID", "9999");
        }
        let private_key = ed25519::PrivateKey::from_seed(0xcadfefe);

        let create_post_json = r#"{"signature":"create_post","posttitle":"second post", "postcontent": "hello world"}"#;
        let create_post_tx_data = TransactionData {
            transaction_type: TransactionType::Call,
            calldata: (SiteCall {
                site_id: Sha256Digest::from_u64(0),
                args: create_post_json.as_bytes().to_vec(),
            })
            .encode()
            .to_vec(),
        };

        let calldata_hash = create_post_tx_data.calculate_hash();
        let create_post_tx = Transaction {
            pub_key: private_key.public_key(),
            signature: private_key.sign_hash(calldata_hash),
            calldata: create_post_tx_data.encode(),
            pow_nonce: 2,
        };

        let execution = Execution::new();
        assert!(execution.verify_transaction(&create_post_tx));
    }
}
