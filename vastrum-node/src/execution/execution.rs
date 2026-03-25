pub struct Execution {
    seen_pow_hash: HashSet<Sha256Digest>,
    seen_pow_hash_by_height: HashMap<u64, Vec<Sha256Digest>>,
    current_block_height: u64,
    pub vastrum_host: VastrumHost,
    pub block_timestamp: u64,
    pub message_sender: ed25519::PublicKey,
    pub db: Arc<BatchDb>,
    state_tree: StateTree,
}
impl Execution {
    #[cfg(not(madsim))]
    pub fn execute_block(&mut self, finalized: FinalizedBlock) {
        self.current_block_height = finalized.block.height;
        self.block_timestamp = finalized.block.timestamp;

        let txs = &finalized.block.transactions;

        let all_signatures_valid = parallel_batch_verifier::verify_signatures(txs);
        let all_pow_valid = self.verify_all_pow(txs);
        let all_gas_limits_valid = Self::verify_all_gas(txs);
        let all_txs_valid_in_block = all_signatures_valid && all_pow_valid && all_gas_limits_valid;

        //as honest leader should ensure all pow hash and sigs are ok
        //if any invalid signature or pow, then skip executing block
        if !all_txs_valid_in_block {
            tracing::warn!("one or more transactions in block failed verification, rejecting block");
        } else {
            let decoded_txs = decompress_and_decode_transactions(txs);
            //cache of contracts transactions in this block touches
            let module_cache = self.preload_modules(&decoded_txs);
            for decoded_tx in decoded_txs {
                self.mark_pow_as_spent(decoded_tx.pow_hash);
                let Some(transaction_data) = decoded_tx.transaction_data else {
                    tracing::warn!("failed to decompress transaction calldata");
                    continue;
                };
                self.message_sender = decoded_tx.pub_key;
                self.execute_transaction(transaction_data, decoded_tx.tx_hash, &module_cache);
            }
        }
        self.prune_spent_pow_hashes();
        //comment out for benchmark
        indexer::index_finalized_block(&self.db, &finalized);
        self.db.write_block(finalized.clone());
        self.db.write_latest_height(finalized.block.height);
        self.db.write_keyvalue_history_to_db(finalized.block.height, KV_RETENTION_WINDOW);
        self.state_tree.write_state_updates_to_jmt_proof_db(&self.db, finalized.block.height);
        self.db.commit();
    }
    #[cfg(not(madsim))]
    fn execute_transaction(
        &mut self,
        transaction_data: TransactionData,
        tx_hash: Sha256Digest,
        module_cache: &HashMap<PathBuf, Module>,
    ) {
        let calldata = transaction_data.calldata;

        if transaction_data.transaction_type == TransactionType::Call {
            self.execute_call_tx(calldata, module_cache);
        } else if transaction_data.transaction_type == TransactionType::DeployNewModule {
            self.execute_deploy_new_module_tx(calldata, tx_hash);
        } else if transaction_data.transaction_type == TransactionType::AddModule {
            self.execute_add_module_tx(calldata);
        } else if transaction_data.transaction_type == TransactionType::DeployStoredModule {
            self.execute_deploy_stored_module_tx(calldata, tx_hash);
        } else if transaction_data.transaction_type == TransactionType::RegisterDomain {
            self.register_domain(calldata);
        }

        //comment out for benchmark
        self.db.set_tx_as_included(tx_hash);
    }

    #[cfg(not(madsim))]
    fn preload_modules(&self, decoded_txs: &[DecodedTx]) -> HashMap<PathBuf, Module> {
        //deduplicate wasm module loads for transactions touching same module
        let mut paths_to_preload: HashSet<PathBuf> = HashSet::new();
        for decoded_tx in decoded_txs {
            let Some(transaction_data) = &decoded_tx.transaction_data else { continue };
            if transaction_data.transaction_type == TransactionType::Call {
                if let Ok(site_call) = borsh::from_slice::<SiteCall>(&transaction_data.calldata) {
                    if let Some(site_data) = self.db.read_site(site_call.site_id) {
                        let path = self.db.calculate_module_file_path(site_data.module_id);
                        if path.exists() {
                            paths_to_preload.insert(path);
                        }
                    }
                }
            }
        }
        let engine = self.vastrum_host.engine();
        let preloaded_modules = paths_to_preload
            .into_par_iter()
            .filter_map(|p| {
                let module = unsafe { Module::deserialize_file(engine, &p) }.ok()?;
                Some((p, module))
            })
            .collect();
        return preloaded_modules;
    }

    pub fn verify_pow(&self, transaction: &Transaction) -> bool {
        return self.verify_pow_threshold(transaction)
            && self.verify_pow_not_spent(transaction)
            && self.verify_pow_not_expired(transaction)
            && self.verify_pow_not_in_future(transaction);
    }

    #[cfg(not(madsim))]
    fn verify_all_pow(&self, transactions: &[Transaction]) -> bool {
        let pow_hash_not_used_before = transactions.par_iter().all(|tx| self.verify_pow(tx));

        let no_duplicate_pow_hash_in_block = {
            let mut seen = HashSet::with_capacity(transactions.len());
            transactions.iter().all(|tx| seen.insert(tx.calculate_pow_hash()))
        };

        let valid_pow = pow_hash_not_used_before && no_duplicate_pow_hash_in_block;
        return valid_pow;
    }

    fn verify_all_gas(transactions: &[Transaction]) -> bool {
        for tx in transactions {
            let has_pow_for_gas_limit = tx.verify_gas();
            if !has_pow_for_gas_limit {
                return false;
            }
        }
        return true;
    }

    fn verify_pow_threshold(&self, transaction: &Transaction) -> bool {
        let pow_hash = transaction.calculate_pow_hash();
        return pow_hash < self.pow_threshold();
    }

    fn verify_pow_not_spent(&self, transaction: &Transaction) -> bool {
        let pow_hash = transaction.calculate_pow_hash();
        return self.seen_pow_hash.get(&pow_hash).is_none();
    }

    fn verify_pow_not_expired(&self, transaction: &Transaction) -> bool {
        return transaction.recent_block_height
            >= self.current_block_height.saturating_sub(VALIDITY_WINDOW);
    }

    fn verify_pow_not_in_future(&self, transaction: &Transaction) -> bool {
        return transaction.recent_block_height <= self.current_block_height;
    }

    #[cfg(not(madsim))]
    fn mark_pow_as_spent(&mut self, pow_hash: Sha256Digest) {
        self.seen_pow_hash.insert(pow_hash);
        self.seen_pow_hash_by_height.entry(self.current_block_height).or_default().push(pow_hash);
    }
    fn prune_spent_pow_hashes(&mut self) {
        let expired_height = self.current_block_height.saturating_sub(VALIDITY_WINDOW);
        for (height, hashes) in &self.seen_pow_hash_by_height {
            if *height < expired_height {
                for hash in hashes {
                    self.seen_pow_hash.remove(hash);
                }
            }
        }
        self.seen_pow_hash_by_height.retain(|&height, _| height >= expired_height);
    }

    fn pow_threshold(&self) -> Sha256Digest {
        return Sha256Digest::from([
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        ]);
    }

    pub fn new(db: Arc<Db>) -> Execution {
        return Execution {
            seen_pow_hash: HashSet::new(),
            seen_pow_hash_by_height: HashMap::new(),
            current_block_height: 0,
            vastrum_host: VastrumHost::new(),
            block_timestamp: 0,
            message_sender: ed25519::PublicKey::default(),
            db: BatchDb::new(db),
            state_tree: StateTree::new(),
        };
    }

    pub fn latest_state_root(&self) -> Sha256Digest {
        self.state_tree.latest_state_root()
    }
    #[cfg(madsim)]
    pub fn execute_block(&mut self, finalized: FinalizedBlock) {
        self.current_block_height = finalized.block.height;
        self.block_timestamp = finalized.block.timestamp;
        self.prune_spent_pow_hashes();
        indexer::index_finalized_block(&self.db, &finalized);
        self.db.write_block(finalized.clone());
        self.db.write_latest_height(finalized.block.height);
        self.db.write_keyvalue_history_to_db(finalized.block.height, KV_RETENTION_WINDOW);
        self.state_tree.write_state_updates_to_jmt_proof_db(&self.db, finalized.block.height);
        self.db.commit();
    }

    pub fn restore_from_disk(db: Arc<Db>) -> Execution {
        let latest_finalized_height = db.read_latest_finalized_height();

        // Rebuild the PoW dedup set from the last VALIDITY_WINDOW blocks so we reject replayed transactions on restart
        let mut seen_pow_hash = HashSet::new();
        let mut seen_pow_hash_by_height: HashMap<u64, Vec<Sha256Digest>> = HashMap::new();
        let window_start = latest_finalized_height.saturating_sub(VALIDITY_WINDOW);

        for height in window_start..=latest_finalized_height {
            if height == 0 {
                continue; //skip genesis block
            }
            let finalized = db.read_block(height).expect("finalized block missing from db");
            for tx in &finalized.block.transactions {
                let pow_hash = tx.calculate_pow_hash();
                seen_pow_hash.insert(pow_hash);
                seen_pow_hash_by_height.entry(height).or_default().push(pow_hash);
            }
        }

        let state_tree = StateTree::restore(&db);
        Execution {
            seen_pow_hash,
            seen_pow_hash_by_height,
            current_block_height: latest_finalized_height,
            vastrum_host: VastrumHost::new(),
            block_timestamp: 0,
            message_sender: ed25519::PublicKey::default(),
            state_tree,
            db: BatchDb::new(db),
        }
    }
}

#[cfg(not(madsim))]
struct DecodedTx {
    tx_hash: Sha256Digest,
    pow_hash: Sha256Digest,
    pub_key: ed25519::PublicKey,
    transaction_data: Option<TransactionData>,
}

#[cfg(not(madsim))]
fn decompress_and_decode_transactions(transactions: &[Transaction]) -> Vec<DecodedTx> {
    transactions
        .par_iter()
        .map(|tx| {
            let decompressed = decompress_calldata(&tx.calldata).ok();
            let transaction_data;
            //try to decode
            if let Some(decompressed) = decompressed {
                transaction_data = borsh::from_slice::<TransactionData>(&decompressed).ok();
            } else {
                transaction_data = None;
            };
            DecodedTx {
                tx_hash: tx.calculate_txhash(),
                pow_hash: tx.calculate_pow_hash(),
                pub_key: tx.pub_key,
                transaction_data,
            }
        })
        .collect()
}
use super::state_tree::StateTree;
use crate::block_indexer::indexer;
use crate::{
    consensus::types::FinalizedBlock,
    db::{BatchDb, Db},
    execution::wasmhost::host::VastrumHost,
};
use vastrum_shared_types::{
    crypto::{ed25519, sha256::Sha256Digest},
    limits::{KV_RETENTION_WINDOW, VALIDITY_WINDOW},
    types::execution::transaction::Transaction,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
#[cfg(not(madsim))]
use {
    super::parallel_batch_verifier,
    rayon::prelude::*,
    vastrum_shared_types::{
        transactioning::compression::decompress_calldata,
        types::application::{
            sitecall::SiteCall,
            transactiondata::{TransactionData, TransactionType},
        },
    },
    std::path::PathBuf,
    wasmtime::Module,
};

#[cfg(test)]
#[path = "execution_tests.rs"]
mod tests;
