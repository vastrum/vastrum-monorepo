use vastrum_contract_macros::{constructor, contract_methods, contract_state};

#[contract_state]
struct Contract {}

#[contract_methods]
impl Contract {
    #[constructor]
    pub fn new(brotli_html_content: Vec<u8>) -> Self {
        runtime::register_static_route("", &brotli_html_content);
        return Self::default();
    }
}
