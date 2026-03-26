mod contract_gen;
mod contract_type;
mod utils;

use proc_macro::TokenStream;

/// Attribute macro for contract state struct.
#[proc_macro_attribute]
pub fn contract_state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    contract_gen::generate_state(item).into()
}

/// Attribute macro for contract methods impl block.
#[proc_macro_attribute]
pub fn contract_methods(_attr: TokenStream, item: TokenStream) -> TokenStream {
    contract_gen::generate_impl(item).into()
}

/// Marker attribute for methods that require authentication.
/// When present, the generated ABI client will use `make_authenticated_call`
/// and require an `account_private_key` parameter.
#[proc_macro_attribute]
pub fn authenticated(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Marker attribute for the constructor method.
/// The constructor is called once when a new site is deployed
#[proc_macro_attribute]
pub fn constructor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Attribute macro for contract types (structs/enums used in contract state or methods).
/// Automatically derives BorshSerialize, BorshDeserialize, Clone, and Default.
#[proc_macro_attribute]
pub fn contract_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    contract_type::contract_type(item)
}
