mod typescript;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(TsType, attributes(ts))]
pub fn derive_ts_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

struct Directions {
    into_wasm_abi: bool,
    from_wasm_abi: bool,
}

fn expand(input: &DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let declaration = typescript::declaration(input)?;
    let directions = parse_directions(input)?;

    let name = &input.ident;
    let declaration_name = name.to_string();
    let js_type = format_ident!("JsTypeFor{}", name);

    let section = declaration.clone();

    let mut generated = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
        const TS_APPEND_CONTENT: &'static str = #section;

        #[wasm_bindgen::prelude::wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(typescript_type = #declaration_name)]
            pub type #js_type;
        }

        fn to_js_value(value: &#name) -> wasm_bindgen::JsValue {
            use wasm_bindgen::UnwrapThrowExt;
            serde_wasm_bindgen::to_value(value).unwrap_throw()
        }

        impl wasm_bindgen::describe::WasmDescribe for #name {
            #[inline]
            fn describe() {
                <#js_type as wasm_bindgen::describe::WasmDescribe>::describe()
            }
        }

        impl wasm_bindgen::describe::WasmDescribeVector for #name {
            #[inline]
            fn describe_vector() {
                <#js_type as wasm_bindgen::describe::WasmDescribeVector>::describe_vector()
            }
        }
    };

    if directions.into_wasm_abi {
        generated.extend(quote! {
            impl wasm_bindgen::convert::IntoWasmAbi for #name {
                type Abi = <#js_type as wasm_bindgen::convert::IntoWasmAbi>::Abi;

                #[inline]
                fn into_abi(self) -> Self::Abi {
                    let typed: #js_type =
                        wasm_bindgen::JsCast::unchecked_into(to_js_value(&self));
                    wasm_bindgen::convert::IntoWasmAbi::into_abi(typed)
                }
            }

            impl wasm_bindgen::convert::OptionIntoWasmAbi for #name {
                #[inline]
                fn none() -> Self::Abi {
                    <#js_type as wasm_bindgen::convert::OptionIntoWasmAbi>::none()
                }
            }


            impl From<#name> for wasm_bindgen::JsValue {
                #[inline]
                fn from(value: #name) -> Self {
                    to_js_value(&value)
                }
            }

            impl wasm_bindgen::convert::VectorIntoWasmAbi for #name {
                type Abi = <#js_type as wasm_bindgen::convert::VectorIntoWasmAbi>::Abi;

                #[inline]
                fn vector_into_abi(vector: Box<[Self]>) -> Self::Abi {
                    let values: Box<[wasm_bindgen::JsValue]> =
                        vector.iter().map(to_js_value).collect();
                    wasm_bindgen::convert::VectorIntoWasmAbi::vector_into_abi(values)
                }
            }
        });
    }

    if directions.from_wasm_abi {
        generated.extend(quote! {
            fn from_js_value(value: &wasm_bindgen::JsValue) -> #name {
                use wasm_bindgen::UnwrapThrowExt;
                serde_wasm_bindgen::from_value(value.clone()).unwrap_throw()
            }

            impl wasm_bindgen::convert::FromWasmAbi for #name {
                type Abi = <#js_type as wasm_bindgen::convert::FromWasmAbi>::Abi;

                #[inline]
                unsafe fn from_abi(js: Self::Abi) -> Self {
                    let typed = unsafe {
                        <#js_type as wasm_bindgen::convert::FromWasmAbi>::from_abi(js)
                    };
                    from_js_value(AsRef::<wasm_bindgen::JsValue>::as_ref(&typed))
                }
            }

            impl wasm_bindgen::convert::OptionFromWasmAbi for #name {
                #[inline]
                fn is_none(abi: &Self::Abi) -> bool {
                    <#js_type as wasm_bindgen::convert::OptionFromWasmAbi>::is_none(abi)
                }
            }

            impl wasm_bindgen::convert::RefFromWasmAbi for #name {
                type Abi = <#js_type as wasm_bindgen::convert::RefFromWasmAbi>::Abi;
                type Anchor = Box<#name>;

                #[inline]
                unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let anchor = unsafe {
                        <#js_type as wasm_bindgen::convert::RefFromWasmAbi>::ref_from_abi(js)
                    };
                    let typed: &#js_type = core::ops::Deref::deref(&anchor);
                    Box::new(from_js_value(AsRef::<wasm_bindgen::JsValue>::as_ref(typed)))
                }
            }

            impl wasm_bindgen::convert::VectorFromWasmAbi for #name {
                type Abi = <#js_type as wasm_bindgen::convert::VectorFromWasmAbi>::Abi;

                #[inline]
                unsafe fn vector_from_abi(js: Self::Abi) -> Box<[Self]> {
                    let values = unsafe {
                        <wasm_bindgen::JsValue as wasm_bindgen::convert::VectorFromWasmAbi>
                            ::vector_from_abi(js)
                    };
                    values.iter().map(from_js_value).collect()
                }
            }
        });
    }

    Ok(quote! {
        #[automatically_derived]
        const _: () = {
            #generated
        };
    })
}

fn parse_directions(input: &DeriveInput) -> Result<Directions, syn::Error> {
    let mut directions = Directions { into_wasm_abi: false, from_wasm_abi: false };
    for attribute in &input.attrs {
        if !attribute.path().is_ident("ts") {
            continue;
        }
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("into_wasm_abi") {
                directions.into_wasm_abi = true;
                return Ok(());
            }
            if meta.path.is_ident("from_wasm_abi") {
                directions.from_wasm_abi = true;
                return Ok(());
            }
            Err(meta.error("expected `into_wasm_abi` or `from_wasm_abi`"))
        })?;
    }
    Ok(directions)
}
