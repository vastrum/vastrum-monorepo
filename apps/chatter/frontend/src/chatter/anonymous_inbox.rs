//TODO need faster sync, optimistically request in batches of 100 or even 1000? binary search to find current tip?
pub struct AnonInbox<T> {
    inbox_content: Vec<String>,
    current_nonce: u64,
    root_hash: Sha256Digest,
    public_key: x25519::PublicKey,
    phantom_t: PhantomData<T>,
}
impl<T> AnonInbox<T>
where
    T: Serialize,
    T: for<'a> Deserialize<'a>,
{
    pub fn import_inbox(public_key: x25519::PublicKey, root_hash: Sha256Digest) -> AnonInbox<T> {
        let inbox = AnonInbox {
            inbox_content: vec![],
            current_nonce: 0,
            public_key,
            root_hash,
            phantom_t: PhantomData,
        };
        return inbox;
    }

    //Syncs data
    //Calculates the inbox_id for each nonce
    //Then checks if there is a message in the inbox_id
    //Continue checking until find an empty inbox_id
    async fn sync_inbox(&mut self, contract: &ContractAbiClient) {
        loop {
            let id = self.get_inbox_id_for_nonce(self.current_nonce);
            let inbox_content = contract.state().await.inbox.get(&id.to_string()).await;
            let is_empty = inbox_content.is_none();

            if is_empty {
                break;
            } else {
                self.inbox_content.push(inbox_content.unwrap());
                self.current_nonce += 1;
            }
        }
    }

    //Each inbox_id is derived from a starting root_hash
    //to get id 5, the root_hash is hashed 5 times
    //to break the hash chain, the hash for id is then concatenated with the namespace for this repository
    //which means only knowing public inbox_id you cannot get next inbox_id for this inbox
    //you have to know private preimage in order to derive next inbox_id
    fn get_inbox_id_for_nonce(&self, id: u64) -> Sha256Digest {
        let mut cur_hash = self.root_hash;
        for _ in 0..id {
            cur_hash = sha256_hash(&cur_hash.encode());
        }
        let hash_bytes = cur_hash.encode();
        let salt_bytes = b"VASTRUM_CHATTER".to_vec();
        let bytes = [hash_bytes, salt_bytes].concat();
        let inbox_id = sha256_hash(&bytes);
        return inbox_id;
    }

    pub async fn send_message_to_inbox(&mut self, message: T, contract: &ContractAbiClient) {
        let json = serde_json::to_string(&message).unwrap();
        self.write_to_key_store(json, contract).await;
    }

    //writes to
    async fn write_to_key_store(&mut self, content: String, contract: &ContractAbiClient) {
        //sync current repository nonce in case new messages sent to this repository
        self.sync_inbox(contract).await;

        let public_key = self.public_key;
        let private_key = x25519::PrivateKey::from_rng();
        let cipher_text = encrypt_string_x25519(&content, &private_key, public_key);
        let inbox_id = self.get_inbox_id_for_nonce(self.current_nonce);

        let content = serde_json::to_string(&cipher_text).unwrap();
        contract.write_to_inbox(inbox_id.to_string(), content).await;
        self.inbox_content.push(serde_json::to_string(&cipher_text).unwrap());
        self.current_nonce += 1;
    }
}

pub struct ReadableAnonInbox<T> {
    inbox: AnonInbox<T>,
    private_key: x25519::PrivateKey,
}
impl<T> ReadableAnonInbox<T>
where
    T: Serialize,
    T: for<'a> Deserialize<'a>,
{
    //creates personal mailbox based on private salt for this site_id
    //uses namespace to allow several personal mailboxes
    //uses additional namespacing for private_key and root_hash to ensure different values
    //as root_hash can be shared to make the mailbox writeable
    //however private_key must be kept private if do not want to share read access
    //should be secure?
    pub async fn create_mailbox(namespace: &str) -> ReadableAnonInbox<T> {
        let private_key = get_private_salt(format!("private_key{namespace}")).await;
        let private_key = x25519::PrivateKey::from_sha256_hash(private_key);

        let public_key = private_key.public_key();
        let inbox = AnonInbox {
            inbox_content: vec![],
            current_nonce: 0,
            public_key,
            root_hash: get_private_salt(namespace.to_string()).await,
            phantom_t: PhantomData,
        };
        let readable_inbox = ReadableAnonInbox { inbox, private_key };
        return readable_inbox;
    }

    pub fn import_mailbox(
        private_key: x25519::PrivateKey,
        root_hash: Sha256Digest,
    ) -> ReadableAnonInbox<T> {
        let public_key = private_key.public_key();
        let inbox = AnonInbox {
            inbox_content: vec![],
            current_nonce: 0,
            public_key,
            root_hash,
            phantom_t: PhantomData,
        };
        let readable_inbox = ReadableAnonInbox { inbox, private_key };
        return readable_inbox;
    }

    pub async fn send_message_to_inbox(&mut self, message: T, contract: &ContractAbiClient) {
        self.inbox.send_message_to_inbox(message, contract).await;
    }

    pub async fn get_messages_in_inbox(&mut self, contract: &ContractAbiClient) -> Vec<T> {
        self.inbox.sync_inbox(contract).await;

        let mut result = vec![];
        for message in self.inbox.inbox_content.iter() {
            let Ok(cipher_text): Result<CipherText, _> = serde_json::from_str(message) else {
                continue;
            };
            let decrypted = decrypt_string_x25519(cipher_text, &self.private_key);
            let Ok(value): Result<T, _> = serde_json::from_str(&decrypted) else {
                continue;
            };
            result.push(value);
        }
        return result;
    }
}

use crate::chatter::chatter_state::ContractAbiClient;
use serde::{Deserialize, Serialize};
use vastrum_shared_types::{
    borsh::BorshExt,
    crypto::{
        encryption::{CipherText, decrypt_string_x25519, encrypt_string_x25519},
        sha256::{Sha256Digest, sha256_hash},
        x25519::{self},
    },
};
use std::marker::PhantomData;
use vastrum_frontend_lib::get_private_salt;
