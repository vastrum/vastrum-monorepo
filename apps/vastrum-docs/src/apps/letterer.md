# Letterer

[Letterer](https://yozq5azfm26qi3vceclwz57fg2727yhqi6ccha5khhnp2uepqj7a.vastrum.net)

Letterer supports
-   Creating documents and saving them to Vastrum
-   Sharing documents with others by sharing a invite link.

Whenever a new document is created in letterer, a private key is generated for that document. 

When this document is saved using save_document() it will be encrypted using that key.

To share your document with others you send them a link containing that private key which allows them to decrypt and see the document.

The private key to the document is also stored encrypted inside user_data in each users "keychain", this allows for saving access to documents across sessions.



```rust
#[contract_type]
struct DocumentWriteOperation {
    content: Vec<u8>,
    metadata: Vec<u8>,
}
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
```

[Letterer on Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/apps/letterer)
