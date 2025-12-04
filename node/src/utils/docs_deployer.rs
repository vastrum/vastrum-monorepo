pub fn deploy_docs_transactions() -> Vec<Transaction> {
    let transactions = deploy_static_site("../apps/static-vastrum-docs".to_string());
    return transactions;
}

use crate::utils::static_deployer::deploy_static_site;
use shared_types::types::execution::transaction::Transaction;
