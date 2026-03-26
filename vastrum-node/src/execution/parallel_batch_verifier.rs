use rayon::prelude::*;
use vastrum_shared_types::{borsh::BorshExt, types::execution::transaction::Transaction};

pub fn verify_signatures(transactions: &[Transaction]) -> bool {
    if transactions.is_empty() {
        return true;
    }
    let chunk_size = 16;

    let all_signatures_valid = transactions.par_chunks(chunk_size).all(|chunk| {
        let mut verifier = ed25519_consensus::batch::Verifier::new();
        for transaction in chunk {
            let verification_key = transaction.pub_key.verifying_key();
            let signature = transaction.signature.inner();
            let hash = transaction.calculate_calldata_hash().encode();
            verifier.queue((verification_key, signature, &hash));
        }
        verifier.verify(rand_core_06::OsRng).is_ok()
    });
    return all_signatures_valid;
}
