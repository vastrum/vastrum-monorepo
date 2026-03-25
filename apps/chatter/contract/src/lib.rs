use vastrum_contract_macros::{constructor, contract_methods, contract_state};
use vastrum_runtime_lib::KvMap;

#[contract_state]
struct Contract {
    inbox: KvMap<String, String>,
}

#[contract_methods]
impl Contract {
    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        Self::default()
    }

    pub fn write_to_inbox(&mut self, inbox_id: String, content: String) {
        self.inbox.set(&inbox_id, content);
    }
}
