use crate::utils::{calculate_function_selector, to_pascal_case};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{FnArg, ImplItem, ItemImpl, ItemStruct, Pat, Type, Visibility};

/// Parsed method information for contract code generation
struct ParsedMethod {
    method_name: syn::Ident,
    param_fields: Vec<TokenStream2>,
    param_names: Vec<syn::Ident>,
}

pub fn generate_state(item: TokenStream) -> TokenStream2 {
    let input_struct: ItemStruct = match syn::parse(item) {
        Ok(s) => s,
        Err(e) => return e.to_compile_error(),
    };

    // Validate named fields
    if !matches!(&input_struct.fields, syn::Fields::Named(_)) {
        return syn::Error::new_spanned(
            &input_struct,
            "contract_state requires a struct with named fields",
        )
        .to_compile_error();
    }

    let struct_name = &input_struct.ident;

    quote! {
        use vastrum_runtime_lib::runtime;

        #[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Default)]
        #input_struct

        impl #struct_name {
            fn __load() -> Self {
                let bytes = runtime::kv_get("__state");
                borsh::from_slice(&bytes).unwrap()
            }

            fn __save(&self) {
                runtime::kv_insert("__state", &borsh::to_vec(self).unwrap());
            }
        }
    }
}

pub fn generate_impl(item: TokenStream) -> TokenStream2 {
    let input_impl: ItemImpl = match syn::parse(item) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };

    // Get the struct name from the impl
    let struct_name = match &*input_impl.self_ty {
        Type::Path(type_path) => type_path.path.segments.last().unwrap().ident.clone(),
        _ => {
            return syn::Error::new_spanned(&input_impl.self_ty, "expected a struct type")
                .to_compile_error();
        }
    };

    // Extract public methods
    let mut pub_methods = Vec::new();
    for item in &input_impl.items {
        if let ImplItem::Fn(method) = item {
            if matches!(method.vis, Visibility::Public(_)) {
                pub_methods.push(method.clone());
            }
        }
    }

    // Parse and validate functions and constructor
    let mut parsed_methods = Vec::new();
    let mut parsed_constructor: Option<ParsedMethod> = None;

    for method in &pub_methods {
        let method_name = method.sig.ident.clone();
        let is_constructor = method.attrs.iter().any(|a| a.path().is_ident("constructor"));

        let mut has_self = false;
        let mut param_fields = Vec::new();
        let mut param_names = Vec::new();

        for arg in &method.sig.inputs {
            match arg {
                FnArg::Receiver(_) => {
                    has_self = true;
                }
                FnArg::Typed(pat_type) => {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = &pat_ident.ident;
                        let param_type = &*pat_type.ty;

                        param_fields.push(quote! {
                            #param_name: #param_type
                        });
                        param_names.push(param_name.clone());
                    } else {
                        return syn::Error::new_spanned(
                            &pat_type.pat,
                            "contract methods only support simple `name: Type` parameters",
                        )
                        .to_compile_error();
                    }
                }
            }
        }

        if is_constructor {
            if has_self {
                return syn::Error::new_spanned(
                    &method.sig,
                    "constructor must be an associated function (`fn(...) -> Self`), not take self",
                )
                .to_compile_error();
            }
            if parsed_constructor.is_some() {
                return syn::Error::new_spanned(
                    &input_impl,
                    "at most one method may have #[constructor]",
                )
                .to_compile_error();
            }
            parsed_constructor = Some(ParsedMethod { method_name, param_fields, param_names });
        } else {
            // Regular methods have to take self, reject static methods
            if !has_self {
                return syn::Error::new_spanned(
                    &method.sig,
                    "contract methods must take &self or &mut self",
                )
                .to_compile_error();
            }

            parsed_methods.push(ParsedMethod { method_name, param_fields, param_names });
        }
    }

    // Generate code
    let mut param_structs = Vec::new();
    let mut handler_fns = Vec::new();
    let mut match_arms = Vec::new();

    //generate handles for external functions
    for method in &parsed_methods {
        let method_name = &method.method_name;
        let method_name_str = method_name.to_string();
        let param_fields = &method.param_fields;
        let param_names = &method.param_names;

        let params_struct_name =
            format_ident!("__External{}Params", to_pascal_case(&method_name_str));
        let handler_name = format_ident!("__external_handler_{}", method_name);

        let param_struct = quote! {
            #[derive(borsh::BorshDeserialize)]
            struct #params_struct_name {
                #(#param_fields),*
            }
        };
        param_structs.push(param_struct);

        let handler_fn = quote! {
            fn #handler_name(params_bytes: &[u8]) {
                let mut contract = #struct_name::__load();
                let params: #params_struct_name = borsh::from_slice(params_bytes).unwrap();
                contract.#method_name(#(params.#param_names),*);
                contract.__save();
            }
        };
        handler_fns.push(handler_fn);

        let sel = calculate_function_selector(&method_name_str);
        let match_arm = quote! {
            [#(#sel),*] => #handler_name(params)
        };
        match_arms.push(match_arm);
    }

    // Generate constructor entrypoint
    let constructor_code = if let Some(ctor) = &parsed_constructor {
        let method_name = &ctor.method_name;
        let method_name_str = method_name.to_string();
        let param_fields = &ctor.param_fields;
        let param_names = &ctor.param_names;

        let params_struct_name =
            format_ident!("__External{}Params", to_pascal_case(&method_name_str));

        param_structs.push(quote! {
            #[derive(borsh::BorshDeserialize)]
            struct #params_struct_name {
                #(#param_fields),*
            }
        });

        if param_names.is_empty() {
            quote! {
                #[unsafe(no_mangle)]
                pub extern "C" fn construct(ptr: *const u8, len: u32) {
                    __setup_panic_hook();
                    let contract = #struct_name::#method_name();
                    contract.__save();
                }
            }
        } else {
            quote! {
                #[unsafe(no_mangle)]
                pub extern "C" fn construct(ptr: *const u8, len: u32) {
                    __setup_panic_hook();
                    let params_bytes = if len == 0 { &[] as &[u8] } else {
                        unsafe { core::slice::from_raw_parts(ptr, len as usize) }
                    };
                    let params: #params_struct_name = borsh::from_slice(params_bytes).unwrap();
                    let contract = #struct_name::#method_name(#(params.#param_names),*);
                    contract.__save();
                }
            }
        }
    } else {
        // Auto generate default constructor
        quote! {
            #[unsafe(no_mangle)]
            pub extern "C" fn construct(ptr: *const u8, len: u32) {
                __setup_panic_hook();
                let contract = #struct_name::default();
                contract.__save();
            }
        }
    };

    // Generate dispatch function
    let dispatch_fn = quote! {
        fn dispatch(input: &[u8]) {
            let selector = &input[0..8];
            let params = &input[8..];
            match selector {
                #(#match_arms,)*
                _ => panic!("Unknown selector")
            }
        }
    };

    // Generate allocator for host to use when returning variable-length data
    let alloc_fn = quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn __alloc(size: u32) -> *mut u8 {
            assert!(size > 0, "zero-size allocation");
            unsafe {
                let layout = core::alloc::Layout::from_size_align(size as usize, 1).unwrap();
                let result = std::alloc::alloc(layout);
                assert!(!result.is_null(), "allocation failed");
                result
            }
        }
    };

    // Generate panic hook setup
    let panic_hook = quote! {
        fn __setup_panic_hook() {
            use std::sync::Once;
            static HOOK: Once = Once::new();
            HOOK.call_once(|| {
                std::panic::set_hook(Box::new(|info| {
                    let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = info.payload().downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic".to_string()
                    };
                    let location = info.location()
                        .map(|l| format!(" at {}:{}:{}", l.file(), l.line(), l.column()))
                        .unwrap_or_default();
                    runtime::log(&format!("PANIC: {}{}", msg, location));
                }));
            });
        }
    };

    // Generate WASM entry point
    let makecall_fn = quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn makecall(ptr: *const u8, len: u32) {
            __setup_panic_hook();
            let input = unsafe { core::slice::from_raw_parts(ptr, len as usize) };
            dispatch(input);
        }
    };

    // All generated code at crate level
    quote! {
        #alloc_fn

        #input_impl

        #(#param_structs)*

        #(#handler_fns)*

        #dispatch_fn

        #panic_hook

        #makecall_fn

        #constructor_code
    }
}
