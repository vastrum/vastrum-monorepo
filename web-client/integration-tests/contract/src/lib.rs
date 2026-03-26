use vastrum_contract_macros::{authenticated, constructor, contract_methods, contract_state};
use vastrum_runtime_lib::KvMap;

#[contract_state]
struct Contract {
    data: KvMap<String, String>,
}

#[contract_methods]
impl Contract {
    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        Self::default()
    }

    #[authenticated]
    pub fn set_data(&mut self, key: String, value: String) {
        self.data.set(&key, value);
    }
}
