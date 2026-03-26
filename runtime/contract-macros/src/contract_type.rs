use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse_macro_input};

pub fn contract_type(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);

    match item {
        Item::Enum(mut e) => {
            // Add #[default] to first variant
            if let Some(first_variant) = e.variants.first_mut() {
                first_variant.attrs.push(syn::parse_quote!(#[default]));
            }
            quote! {
                #[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone, Default)]
                #e
            }
            .into()
        }
        _ => quote! {
            #[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Clone, Default)]
            #item
        }
        .into(),
    }
}
