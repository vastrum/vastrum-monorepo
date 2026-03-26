#[contract_state]
struct Contract {
    user_data: KvMap<Ed25519PublicKey, Vec<u8>>,
    documents: KvMap<Ed25519PublicKey, Vec<u8>>,
    doc_metadata: KvMap<Ed25519PublicKey, Vec<u8>>,
}

#[contract_methods]
impl Contract {
    #[authenticated]
    pub fn save_user_data(&mut self, data: Vec<u8>) {
        let sender = message_sender();
        self.user_data.set(&sender, data);
    }

    #[authenticated]
    pub fn save_document(
        &mut self,
        document_key: Ed25519PublicKey,
        signature: Ed25519Signature,
        operation: DocumentWriteOperation,
    ) {
        let encoded = borsh::to_vec(&operation).unwrap();
        let hash = runtime::sha256(&encoded);
        let signature_matches_document_key = document_key.verify(&hash, &signature);
        if !signature_matches_document_key {
            return;
        }
        self.documents.set(&document_key, operation.content);
        self.doc_metadata.set(&document_key, operation.metadata);
    }

    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        return Self::default();
    }
}
#[contract_type]
struct DocumentWriteOperation {
    content: Vec<u8>,
    metadata: Vec<u8>,
}
use vastrum_contract_macros::{
    authenticated, constructor, contract_methods, contract_state, contract_type,
};
use vastrum_runtime_lib::{
    Ed25519PublicKey, Ed25519Signature, Ed25519Verify, KvMap, runtime::message_sender,
};
