# Multisig Aggregator

Multisig aggregator websites allow for multiple people to manage multisig wallets.


Such a frontend allows for each signer to asynchronously sign a transaction and then post
their signature share to a database where the signature shares eventually get aggregated into a complete signature.

It would be cool if this frontend was completely decentralized.

```rust
#[contract_type]
struct ProposedTransaction {
    //contains full transaction data
    transaction_bytes: Vec<u8>, //maybe structured as EIP-712 transaction or even more comprehensive transaction preview format
    signatures: Vec<Ed25519Signature>,
}

#[contract_type] 
struct MultisigAccount {
    //u64 is timestamp of creation
    proposed_transactions: KvVecBTree<u64, ProposedTransaction>,
}

#[contract_state]
struct Contract {
    multisig_accounts: KvMap<Ed25519PublicKey, MultisigAccount>,
}

#[contract_methods]
impl Contract {
    pub fn propose_transaction(&mut self, multisig_account: Ed25519PublicKey, transaction_bytes: Vec<u8>) {
        let mut account = self.multisig_accounts.get(multisig_account);
        let proposed_tx = ProposedTransaction{ transaction_bytes, vec[]};
        account.proposed_transactions.push(block_time(), proposed_tx);
        self.multisig_accounts.set(multisig_account, account);
    }
    pub fn sign_transaction(&mut self, multisig_account: Ed25519PublicKey, transaction_id: u64, signature : Ed25519Signature) {
        let account = self.multisig_accounts.get(multisig_account);
        let proposed_transaction = account.proposed_transactions.get(transaction_id);
        proposed_transaction.signatures.push(signature);
    }
}
```

You would then develop a frontend to interact with this smartcontract and display pending transactions.

```rust
async fn display_proposed_transactions(account: Ed25519PublicKey) -> Vec<ProposedTransaction> {
    let client = ContractAbiClient::new(MULTISIG_AGGREGATOR_SITE_ID);
    let state = client.state().await;
    let multisig_account = state.multisig_accounts.get(account).await;
    let ten_most_recent_proposed_txs = multisig_account.proposed_transactions.get_descending_entries(10, 0).await;
    return ten_most_recent_proposed_txs;
}
```    


For proposing transactions could use Helios RPC in frontend to display Ethereum balance for example and suggest Ethereum transfer transactions. Ie could create proposed transaction to send 5 ETH to this address.

Potentially could have completely decentralized solution for creating transactions, signing transactions and maybe even executing transactions

Currently the web-client does not have any wallet integration, however should be feasible to add bindings to allow frontends to request signatures of transactions and hashes

It would also be cool if the frontend allows some limited local simulation of what the transaction does.

---
*Using ed25519 pubkeys and signatures types just as examples*

*Skipped authentication for proposing transaction and signing transactions but both are solvable, just a minimal proof of concept*
