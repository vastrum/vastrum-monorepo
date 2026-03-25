use vastrum_contract_macros::{authenticated, constructor, contract_methods, contract_state};
use vastrum_runtime_lib::{Ed25519PublicKey, runtime::message_sender};

#[contract_state]
struct Contract {
    admin: Ed25519PublicKey,
}

#[contract_methods]
impl Contract {
    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>, admin: Ed25519PublicKey) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        Self { admin }
    }

    #[authenticated]
    pub fn set_page(&self, brotli_html_content: Vec<u8>) {
        if message_sender() != self.admin {
            return;
        }
        runtime::register_static_route("", &brotli_html_content);
    }
}
