use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vastrum_runtime_shared::calculate_function_selector;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use syn::{
    Attribute, FnArg, GenericArgument, ImplItem, ItemEnum, ItemImpl, ItemStruct, Pat,
    PathArguments, ReturnType, Type, Visibility,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbiType {
    // Primitives
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    String,

    // Composites
    Option(Box<AbiType>),
    Vec(Box<AbiType>),
    Array { elem: Box<AbiType>, len: usize },
    BTreeMap { key: Box<AbiType>, value: Box<AbiType> },

    // KV types (Vastrum-specific)
    KvMap { key: Box<AbiType>, value: Box<AbiType> },
    KvVec { elem: Box<AbiType> },
    KvBTree { key: Box<AbiType>, value: Box<AbiType> },
    KvVecBTree { sort: Box<AbiType>, value: Box<AbiType> },

    // Domain-specific primitives
    Ed25519PublicKey,
    Ed25519Signature,

    // User-defined type reference
    Defined { name: std::string::String, generics: Vec<AbiType> },

    // Tuple type
    Tuple(Vec<AbiType>),
}

/// Type definition (struct or enum)
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: std::string::String,
    pub generics: Vec<std::string::String>,
    pub ty: TypeDefKind,
}

/// Kind of type definition
#[derive(Debug, Clone)]
pub enum TypeDefKind {
    Struct { fields: DefinedFields },
    Enum { variants: Vec<EnumVariant> },
}

/// Fields in a struct or enum variant
#[derive(Debug, Clone)]
pub enum DefinedFields {
    Named(Vec<FieldInfo>),
    Tuple(Vec<AbiType>),
    Unit,
}

/// Enum variant definition
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: std::string::String,
    pub fields: DefinedFields,
}

impl AbiType {
    /// Returns true if this type is a KV collection (KvMap, KvVec, KvBTree, KvVecBTree).
    fn is_kv_type(&self) -> bool {
        matches!(
            self,
            AbiType::KvMap { .. }
                | AbiType::KvVec { .. }
                | AbiType::KvBTree { .. }
                | AbiType::KvVecBTree { .. }
        )
    }

    /// Convert AbiType to a TokenStream for code generation
    pub fn to_tokens(&self) -> TokenStream {
        match self {
            AbiType::Bool => quote! { bool },
            AbiType::U8 => quote! { u8 },
            AbiType::I8 => quote! { i8 },
            AbiType::U16 => quote! { u16 },
            AbiType::I16 => quote! { i16 },
            AbiType::U32 => quote! { u32 },
            AbiType::I32 => quote! { i32 },
            AbiType::U64 => quote! { u64 },
            AbiType::I64 => quote! { i64 },
            AbiType::U128 => quote! { u128 },
            AbiType::I128 => quote! { i128 },
            AbiType::String => quote! { String },
            AbiType::Option(inner) => {
                let inner_tokens = inner.to_tokens();
                quote! { Option<#inner_tokens> }
            }
            AbiType::Vec(inner) => {
                let inner_tokens = inner.to_tokens();
                quote! { Vec<#inner_tokens> }
            }
            AbiType::Array { elem, len } => {
                let elem_tokens = elem.to_tokens();
                quote! { [#elem_tokens; #len] }
            }
            AbiType::BTreeMap { key, value } => {
                let k = key.to_tokens();
                let v = value.to_tokens();
                quote! { ::std::collections::BTreeMap<#k, #v> }
            }
            AbiType::KvMap { key, value } => {
                let k = key.to_tokens();
                let v = value.to_tokens();
                quote! { KvMap<#k, #v> }
            }
            AbiType::KvVec { elem } => {
                let e = elem.to_tokens();
                quote! { KvVec<#e> }
            }
            AbiType::KvBTree { key, value } => {
                let k = key.to_tokens();
                let v = value.to_tokens();
                quote! { KvBTree<#k, #v> }
            }
            AbiType::KvVecBTree { sort, value } => {
                let s = sort.to_tokens();
                let v = value.to_tokens();
                quote! { KvVecBTree<#s, #v> }
            }
            AbiType::Ed25519PublicKey => {
                quote! { vastrum_abi::__private::vastrum_shared_types::crypto::ed25519::PublicKey }
            }
            AbiType::Ed25519Signature => {
                quote! { vastrum_abi::__private::vastrum_shared_types::crypto::ed25519::Signature }
            }
            AbiType::Defined { name, generics } => {
                let name_ident = format_ident!("{}", name);
                if generics.is_empty() {
                    quote! { #name_ident }
                } else {
                    let mut gen_tokens = Vec::new();
                    for g in generics {
                        gen_tokens.push(g.to_tokens());
                    }
                    quote! { #name_ident<#(#gen_tokens),*> }
                }
            }
            AbiType::Tuple(elems) => {
                if elems.is_empty() {
                    quote! { () }
                } else {
                    let mut elem_tokens = Vec::new();
                    for e in elems {
                        elem_tokens.push(e.to_tokens());
                    }
                    quote! { (#(#elem_tokens),*) }
                }
            }
        }
    }

    /// Convert AbiType to a native type TokenStream (for state reading)
    pub fn to_native_tokens(&self) -> TokenStream {
        match self {
            AbiType::KvMap { key, value } => {
                let k = key.to_tokens();
                let v = value.to_tokens();
                quote! { vastrum_abi::__private::vastrum_native_types::KvMap<#k, #v> }
            }
            AbiType::KvVec { elem } => {
                let e = elem.to_tokens();
                quote! { vastrum_abi::__private::vastrum_native_types::KvVec<#e> }
            }
            AbiType::KvBTree { key, value } => {
                let k = key.to_tokens();
                let v = value.to_tokens();
                quote! { vastrum_abi::__private::vastrum_native_types::KvBTree<#k, #v> }
            }
            AbiType::KvVecBTree { sort, value } => {
                let s = sort.to_tokens();
                let v = value.to_tokens();
                quote! { vastrum_abi::__private::vastrum_native_types::KvVecBTree<#s, #v> }
            }
            AbiType::Defined { .. } => self.to_tokens(),
            _ => self.to_tokens(),
        }
    }
}

// ============================================================================
// syn::Type to AbiType conversion
// ============================================================================

/// Convert a syn::Type to AbiType
pub fn syn_type_to_abi_type(ty: &Type) -> Result<AbiType, std::string::String> {
    match ty {
        Type::Path(type_path) => {
            let segment =
                type_path.path.segments.last().ok_or_else(|| "Empty type path".to_string())?;
            let name = segment.ident.to_string();

            // Handle primitives
            match name.as_str() {
                "bool" => return Ok(AbiType::Bool),
                "u8" => return Ok(AbiType::U8),
                "i8" => return Ok(AbiType::I8),
                "u16" => return Ok(AbiType::U16),
                "i16" => return Ok(AbiType::I16),
                "u32" => return Ok(AbiType::U32),
                "i32" => return Ok(AbiType::I32),
                "u64" => return Ok(AbiType::U64),
                "i64" => return Ok(AbiType::I64),
                "u128" => return Ok(AbiType::U128),
                "i128" => return Ok(AbiType::I128),
                "String" => return Ok(AbiType::String),
                "Ed25519PublicKey" => return Ok(AbiType::Ed25519PublicKey),
                "Ed25519Signature" => return Ok(AbiType::Ed25519Signature),
                _ => {}
            }

            // Handle generic types
            match &segment.arguments {
                PathArguments::None => {
                    // Simple type name
                    Ok(AbiType::Defined { name, generics: vec![] })
                }
                PathArguments::AngleBracketed(args) => {
                    let mut generics = Vec::new();
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            generics.push(syn_type_to_abi_type(inner_ty)?);
                        }
                    }

                    match name.as_str() {
                        "Option" if generics.len() == 1 => {
                            Ok(AbiType::Option(Box::new(generics[0].clone())))
                        }
                        "Vec" if generics.len() == 1 => {
                            Ok(AbiType::Vec(Box::new(generics[0].clone())))
                        }
                        "BTreeMap" if generics.len() == 2 => Ok(AbiType::BTreeMap {
                            key: Box::new(generics[0].clone()),
                            value: Box::new(generics[1].clone()),
                        }),
                        "KvMap" if generics.len() == 2 => Ok(AbiType::KvMap {
                            key: Box::new(generics[0].clone()),
                            value: Box::new(generics[1].clone()),
                        }),
                        "KvVec" if generics.len() == 1 => {
                            Ok(AbiType::KvVec { elem: Box::new(generics[0].clone()) })
                        }
                        "KvBTree" if generics.len() == 2 => Ok(AbiType::KvBTree {
                            key: Box::new(generics[0].clone()),
                            value: Box::new(generics[1].clone()),
                        }),
                        "KvVecBTree" if generics.len() == 2 => Ok(AbiType::KvVecBTree {
                            sort: Box::new(generics[0].clone()),
                            value: Box::new(generics[1].clone()),
                        }),
                        _ => Ok(AbiType::Defined { name, generics }),
                    }
                }
                _ => Ok(AbiType::Defined { name, generics: vec![] }),
            }
        }
        Type::Reference(type_ref) => syn_type_to_abi_type(&type_ref.elem),
        Type::Array(type_array) => {
            let elem = syn_type_to_abi_type(&type_array.elem)?;
            let len = extract_array_len(&type_array.len);
            Ok(AbiType::Array { elem: Box::new(elem), len })
        }
        Type::Tuple(type_tuple) => {
            let mut elems = Vec::new();
            for elem in &type_tuple.elems {
                elems.push(syn_type_to_abi_type(elem)?);
            }
            Ok(AbiType::Tuple(elems))
        }
        _ => {
            // Use quote to get a string representation of the unsupported type
            let type_str = quote! { #ty }.to_string();
            Err(format!("Unsupported type variant: {}", type_str))
        }
    }
}

/// Extract array length from a syn::Expr
fn extract_array_len(expr: &syn::Expr) -> usize {
    match expr {
        syn::Expr::Lit(lit) => {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                int_lit.base10_parse().unwrap_or(0)
            } else {
                0
            }
        }
        _ => 0,
    }
}

// ============================================================================
// Contract Info Structures
// ============================================================================

/// Constructor information
#[derive(Debug, Clone)]
pub struct ConstructorInfo {
    pub params: Vec<FieldInfo>,
}

/// Contract information
#[derive(Debug, Clone)]
pub struct ContractInfo {
    pub state_name: std::string::String,
    pub state_fields: Vec<FieldInfo>,
    pub constructor: ConstructorInfo,
    pub methods: Vec<MethodInfo>,
    pub custom_types: Vec<TypeDef>,
}

/// Field with name and type
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: std::string::String,
    pub ty: AbiType,
}

/// Method information
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: std::string::String,
    pub params: Vec<FieldInfo>,
    pub is_public: bool,
    pub requires_auth: bool,
}

/// User-defined type (struct or enum) from source
pub enum UserType {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

/// Contract definition extracted from source file
pub struct ContractDef {
    pub state: ItemStruct,
    pub methods: ItemImpl,
    pub custom_types: Vec<UserType>,
}

/// Collected types from directory
struct CollectedTypes {
    structs: HashMap<std::string::String, ItemStruct>,
    enums: HashMap<std::string::String, ItemEnum>,
}

/// Recursively collect all structs and enums from .rs files in a directory
fn collect_types_from_dir(dir: &Path) -> CollectedTypes {
    let mut collected = CollectedTypes { structs: HashMap::new(), enums: HashMap::new() };
    collect_types_recursive(dir, &mut collected);
    collected
}

fn collect_types_recursive(dir: &Path, collected: &mut CollectedTypes) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_types_recursive(&path, collected);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(source) = std::fs::read_to_string(&path) {
                if let Ok(file) = syn::parse_file(&source) {
                    for item in file.items {
                        match item {
                            syn::Item::Struct(s) => {
                                if !has_attribute(&s.attrs, "contract_state") {
                                    collected.structs.insert(s.ident.to_string(), s);
                                }
                            }
                            syn::Item::Enum(e) => {
                                collected.enums.insert(e.ident.to_string(), e);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// Parse contract source from src/ directory and extract contract definition
pub fn parse_contract_source(src_dir: &Path) -> Result<ContractDef, std::string::String> {
    let source = std::fs::read_to_string(src_dir.join("lib.rs"))
        .map_err(|e| format!("Failed to read lib.rs: {}", e))?;
    let file: syn::File = syn::parse_file(&source).map_err(|e| format!("Parse error: {}", e))?;

    let mut state_struct = None;
    let mut methods_impl = None;

    for item in &file.items {
        if let syn::Item::Struct(s) = item {
            if has_attribute(&s.attrs, "contract_state") {
                state_struct = Some(s.clone());
            }
        }
        if let syn::Item::Impl(i) = item {
            if has_attribute(&i.attrs, "contract_methods") {
                methods_impl = Some(i.clone());
            }
        }
    }

    let state = state_struct.ok_or("No #[contract_state] found")?;
    let methods = methods_impl.ok_or("No #[contract_methods] found")?;
    let collected = collect_types_from_dir(src_dir);
    let custom_types = extract_referenced_user_types(&state, &methods, &collected);

    Ok(ContractDef { state, methods, custom_types })
}

/// Check if an attribute list contains a specific attribute
fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

/// Known built-in types that should not be considered custom types
const KNOWN_TYPES: &[&str] = &[
    "String",
    "Vec",
    "Option",
    "KvMap",
    "KvVec",
    "KvBTree",
    "KvVecBTree",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "bool",
    "Ed25519PublicKey",
    "Ed25519Signature",
];

/// Extract custom types (structs and enums) that are referenced by the contract
fn extract_referenced_user_types(
    state: &ItemStruct,
    methods: &ItemImpl,
    collected: &CollectedTypes,
) -> Vec<UserType> {
    let mut referenced = HashSet::new();

    // Walk each field in the state struct and collect type names
    if let syn::Fields::Named(fields) = &state.fields {
        for field in &fields.named {
            collect_type_names(&field.ty, &mut referenced);
        }
    }

    // Also collect types from method parameters and return types
    for item in &methods.items {
        if let ImplItem::Fn(method) = item {
            for arg in &method.sig.inputs {
                if let FnArg::Typed(pat_type) = arg {
                    collect_type_names(&pat_type.ty, &mut referenced);
                }
            }
            // Also collect return type
            if let ReturnType::Type(_, ty) = &method.sig.output {
                collect_type_names(ty, &mut referenced);
            }
        }
    }

    // Recursively collect nested types from custom structs and enums
    loop {
        let mut new_types = HashSet::new();
        for type_name in &referenced {
            // Check structs
            if let Some(s) = collected.structs.get(type_name) {
                collect_type_names_from_struct_fields(s, &mut new_types);
            }
            // Check enums
            if let Some(e) = collected.enums.get(type_name) {
                collect_type_names_from_enum_variants(e, &mut new_types);
            }
        }
        let before = referenced.len();
        referenced.extend(new_types);
        if referenced.len() == before {
            break;
        }
    }

    // Filter and collect user-defined types
    let mut result = Vec::new();

    for s in collected.structs.values() {
        let name = s.ident.to_string();
        if referenced.contains(&name) && !KNOWN_TYPES.contains(&name.as_str()) {
            result.push(UserType::Struct(s.clone()));
        }
    }

    for e in collected.enums.values() {
        let name = e.ident.to_string();
        if referenced.contains(&name) && !KNOWN_TYPES.contains(&name.as_str()) {
            result.push(UserType::Enum(e.clone()));
        }
    }

    result
}

/// Collect type names from struct fields
fn collect_type_names_from_struct_fields(s: &ItemStruct, names: &mut HashSet<std::string::String>) {
    match &s.fields {
        syn::Fields::Named(fields) => {
            for field in &fields.named {
                collect_type_names(&field.ty, names);
            }
        }
        syn::Fields::Unnamed(fields) => {
            for field in &fields.unnamed {
                collect_type_names(&field.ty, names);
            }
        }
        syn::Fields::Unit => {}
    }
}

/// Collect type names from enum variants
fn collect_type_names_from_enum_variants(e: &ItemEnum, names: &mut HashSet<std::string::String>) {
    for variant in &e.variants {
        match &variant.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    collect_type_names(&field.ty, names);
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in &fields.unnamed {
                    collect_type_names(&field.ty, names);
                }
            }
            syn::Fields::Unit => {}
        }
    }
}

/// Recursively collect type names from a Type, including generic arguments
fn collect_type_names(ty: &Type, names: &mut HashSet<std::string::String>) {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                names.insert(segment.ident.to_string());

                // Also collect generic type arguments
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            collect_type_names(inner_ty, names);
                        }
                    }
                }
            }
        }
        Type::Reference(type_ref) => {
            collect_type_names(&type_ref.elem, names);
        }
        Type::Array(type_array) => {
            collect_type_names(&type_array.elem, names);
        }
        Type::Tuple(type_tuple) => {
            for elem in &type_tuple.elems {
                collect_type_names(elem, names);
            }
        }
        _ => {}
    }
}

/// Convert ContractDef (syn types) to ContractInfo
pub fn contract_def_to_info(def: &ContractDef) -> ContractInfo {
    let state_name = def.state.ident.to_string();

    // Extract state fields with typed ABI types
    let mut state_fields = Vec::new();
    if let syn::Fields::Named(fields) = &def.state.fields {
        for f in &fields.named {
            let field_name = f.ident.as_ref().unwrap().to_string();
            let ty = syn_type_to_abi_type(&f.ty).unwrap_or_else(|e| {
                panic!("Failed to parse type for state field '{}': {}", field_name, e)
            });
            state_fields.push(FieldInfo { name: field_name, ty });
        }
    }

    // Extract methods and constructor
    let mut methods = Vec::new();
    let mut constructor = None;
    for item in &def.methods.items {
        if let ImplItem::Fn(method) = item {
            let method_name = method.sig.ident.to_string();

            let mut params = Vec::new();
            for arg in &method.sig.inputs {
                if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = pat_ident.ident.to_string();
                        let ty = syn_type_to_abi_type(&pat_type.ty).unwrap_or_else(|e| {
                            panic!(
                                "Failed to parse type for parameter '{}' in method '{}': {}",
                                param_name, method_name, e
                            )
                        });
                        params.push(FieldInfo { name: param_name, ty });
                    }
                }
            }

            if has_attribute(&method.attrs, "constructor") {
                constructor = Some(ConstructorInfo { params });
            } else {
                let requires_auth = has_attribute(&method.attrs, "authenticated");
                methods.push(MethodInfo {
                    name: method_name,
                    params,
                    is_public: matches!(method.vis, Visibility::Public(_)),
                    requires_auth,
                });
            }
        }
    }

    // Default constructor if none declared
    let constructor = constructor.unwrap_or(ConstructorInfo { params: vec![] });

    // Extract custom types (structs and enums)
    let mut custom_types = Vec::new();
    for ut in &def.custom_types {
        custom_types.push(user_type_to_type_def(ut));
    }

    ContractInfo { state_name, state_fields, constructor, methods, custom_types }
}

/// Convert UserType (syn) to TypeDef
fn user_type_to_type_def(ut: &UserType) -> TypeDef {
    match ut {
        UserType::Struct(s) => {
            let name = s.ident.to_string();
            let generics = extract_generics(&s.generics);
            let fields = syn_fields_to_defined_fields(&s.fields);
            TypeDef { name, generics, ty: TypeDefKind::Struct { fields } }
        }
        UserType::Enum(e) => {
            let name = e.ident.to_string();
            let generics = extract_generics(&e.generics);
            let mut variants = Vec::new();
            for v in &e.variants {
                variants.push(EnumVariant {
                    name: v.ident.to_string(),
                    fields: syn_fields_to_defined_fields(&v.fields),
                });
            }
            TypeDef { name, generics, ty: TypeDefKind::Enum { variants } }
        }
    }
}

/// Extract generic parameters from syn::Generics
fn extract_generics(generics: &syn::Generics) -> Vec<std::string::String> {
    let mut result = Vec::new();
    for p in &generics.params {
        if let syn::GenericParam::Type(t) = p {
            result.push(t.ident.to_string());
        }
    }
    result
}

/// Convert syn::Fields to DefinedFields
fn syn_fields_to_defined_fields(fields: &syn::Fields) -> DefinedFields {
    match fields {
        syn::Fields::Named(f) => {
            let mut named = Vec::new();
            for field in &f.named {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let ty = syn_type_to_abi_type(&field.ty).unwrap_or_else(|e| {
                    panic!("Failed to parse type for field '{}': {}", field_name, e)
                });
                named.push(FieldInfo { name: field_name, ty });
            }
            DefinedFields::Named(named)
        }
        syn::Fields::Unnamed(f) => {
            let mut types = Vec::new();
            for (i, field) in f.unnamed.iter().enumerate() {
                let ty = syn_type_to_abi_type(&field.ty).unwrap_or_else(|e| {
                    panic!("Failed to parse type for tuple field {}: {}", i, e)
                });
                types.push(ty);
            }
            DefinedFields::Tuple(types)
        }
        syn::Fields::Unit => DefinedFields::Unit,
    }
}

// ============================================================================
// Code Generation
// ============================================================================

/// Generate ABI client code from ContractInfo
fn generate_abi_code_from_info(contract: &ContractInfo) -> TokenStream {
    let custom_types_code = generate_custom_types_from_info(&contract.custom_types);
    let state_code = generate_field_getters_from_info(&contract.state_name, &contract.state_fields);
    let client_code = generate_client_code(contract);

    quote! {
        use std::sync::Arc;
        pub use vastrum_abi::__private::vastrum_rpc_client::{RpcProvider, SentTxBehavior};

        #custom_types_code
        #state_code
        #client_code
    }
}

/// Generate code for custom types from typed TypeDef info
fn generate_custom_types_from_info(custom_types: &[TypeDef]) -> TokenStream {
    let mut type_defs = Vec::new();

    for typedef in custom_types {
        let ident = format_ident!("{}", typedef.name);

        let def = match &typedef.ty {
            TypeDefKind::Struct { fields } => generate_struct_def(&ident, fields),
            TypeDefKind::Enum { variants } => generate_enum_def(&ident, variants),
        };
        type_defs.push(def);
    }

    quote! { #(#type_defs)* }
}

/// Generate a struct definition
fn generate_struct_def(ident: &proc_macro2::Ident, fields: &DefinedFields) -> TokenStream {
    match fields {
        DefinedFields::Named(named) => {
            if named.is_empty() {
                quote! {
                    #[derive(vastrum_abi::__private::borsh::BorshSerialize, vastrum_abi::__private::borsh::BorshDeserialize, Clone, Debug, Default)]
                    #[borsh(crate = "vastrum_abi::__private::borsh")]
                    pub struct #ident;
                }
            } else {
                let has_kv = named.iter().any(|f| f.ty.is_kv_type());

                let mut field_tokens = Vec::new();
                for f in named {
                    let field_name = format_ident!("{}", f.name);
                    let field_type = f.ty.to_native_tokens();
                    field_tokens.push(quote! { pub #field_name: #field_type });
                }

                if has_kv {
                    quote! {
                        #[derive(vastrum_abi::__private::borsh::BorshDeserialize, Clone, Debug)]
                        #[borsh(crate = "vastrum_abi::__private::borsh")]
                        pub struct #ident { #(#field_tokens),* }
                    }
                } else {
                    quote! {
                        #[derive(vastrum_abi::__private::borsh::BorshSerialize, vastrum_abi::__private::borsh::BorshDeserialize, Clone, Debug, Default)]
                        #[borsh(crate = "vastrum_abi::__private::borsh")]
                        pub struct #ident { #(#field_tokens),* }
                    }
                }
            }
        }
        DefinedFields::Tuple(types) => {
            if types.is_empty() {
                quote! {
                    #[derive(vastrum_abi::__private::borsh::BorshSerialize, vastrum_abi::__private::borsh::BorshDeserialize, Clone, Debug, Default)]
                    #[borsh(crate = "vastrum_abi::__private::borsh")]
                    pub struct #ident;
                }
            } else {
                let mut field_tokens = Vec::new();
                for t in types {
                    let ty = t.to_native_tokens();
                    field_tokens.push(quote! { pub #ty });
                }
                quote! {
                    #[derive(vastrum_abi::__private::borsh::BorshSerialize, vastrum_abi::__private::borsh::BorshDeserialize, Clone, Debug, Default)]
                    #[borsh(crate = "vastrum_abi::__private::borsh")]
                    pub struct #ident ( #(#field_tokens),* );
                }
            }
        }
        DefinedFields::Unit => {
            quote! {
                #[derive(vastrum_abi::__private::borsh::BorshSerialize, vastrum_abi::__private::borsh::BorshDeserialize, Clone, Debug, Default)]
                #[borsh(crate = "vastrum_abi::__private::borsh")]
                pub struct #ident;
            }
        }
    }
}

/// Generate an enum definition
fn generate_enum_def(ident: &proc_macro2::Ident, variants: &[EnumVariant]) -> TokenStream {
    let mut variant_tokens = Vec::new();
    for (i, v) in variants.iter().enumerate() {
        let variant_name = format_ident!("{}", v.name);
        // Add #[default] to first variant for Default derive
        let default_attr = if i == 0 {
            quote! { #[default] }
        } else {
            quote! {}
        };
        let token = match &v.fields {
            DefinedFields::Named(named) => {
                let mut field_tokens = Vec::new();
                for f in named {
                    let field_name = format_ident!("{}", f.name);
                    let field_type = f.ty.to_native_tokens();
                    field_tokens.push(quote! { #field_name: #field_type });
                }
                quote! { #default_attr #variant_name { #(#field_tokens),* } }
            }
            DefinedFields::Tuple(types) => {
                let mut field_tokens = Vec::new();
                for t in types {
                    field_tokens.push(t.to_native_tokens());
                }
                quote! { #default_attr #variant_name ( #(#field_tokens),* ) }
            }
            DefinedFields::Unit => {
                quote! { #default_attr #variant_name }
            }
        };
        variant_tokens.push(token);
    }

    quote! {
        #[derive(vastrum_abi::__private::borsh::BorshSerialize, vastrum_abi::__private::borsh::BorshDeserialize, Clone, Debug, Default, PartialEq, PartialOrd)]
        #[borsh(crate = "vastrum_abi::__private::borsh")]
        pub enum #ident {
            #(#variant_tokens),*
        }
    }
}

/// Generates ABI client for native contract calls
fn generate_client_code(contract: &ContractInfo) -> TokenStream {
    let client_name = format_ident!("{}AbiClient", contract.state_name);
    let native_state_name = format_ident!("Native{}", contract.state_name);

    let mut pub_methods = Vec::new();
    for m in &contract.methods {
        if m.is_public {
            pub_methods.push(m);
        }
    }
    let method_impls = generate_async_methods_from_info(&pub_methods);
    let deploy_method = generate_deploy_method(&contract.constructor);

    quote! {
        pub struct #client_name {
            client: vastrum_abi::__private::vastrum_rpc_client::RpcClient,
        }

        impl #client_name {
            pub fn new(site_id: vastrum_abi::__private::vastrum_shared_types::crypto::sha256::Sha256Digest) -> Self {
                Self {
                    client: vastrum_abi::__private::vastrum_rpc_client::RpcClient::new(site_id),
                }
            }

            pub fn site_id(&self) -> vastrum_abi::__private::vastrum_shared_types::crypto::sha256::Sha256Digest {
                use vastrum_abi::__private::vastrum_rpc_client::RpcProvider;
                self.client.site_id()
            }

            pub fn with_account_key(self, key: vastrum_abi::__private::vastrum_shared_types::crypto::ed25519::PrivateKey) -> Self {
                Self { client: self.client.with_account_key(key) }
            }

            /// Returns the contract state
            pub async fn state(&self) -> #native_state_name {
                StateReader::from_client(Arc::new(self.client.clone())).state().await
            }

            #deploy_method

            #method_impls
        }
    }
}

/// Generates the contract deploy method for the ABI client
fn generate_deploy_method(constructor: &ConstructorInfo) -> TokenStream {
    let has_params = !constructor.params.is_empty();

    if has_params {
        let mut signature_params = Vec::new();
        let mut param_names = Vec::new();
        for p in &constructor.params {
            let name = format_ident!("{}", p.name);
            let ty = p.ty.to_tokens();
            signature_params.push(quote! { #name: #ty });
            param_names.push(name);
        }

        quote! {
            /// Deploys contract, will only return when contract has actually been deployed and polls for inclusion
            /// Can take some time for this function to return because of this
            #[cfg(not(target_arch = "wasm32"))]
            pub async fn deploy(wasm_path: &str, #(#signature_params),*) -> Self {
                let constructor_calldata = vastrum_abi::__private::borsh::to_vec(&(#(#param_names,)*)).unwrap();
                let site_id = vastrum_abi::__private::vastrum_native_lib::deployers::deploy::deploy_module(
                    wasm_path, constructor_calldata
                ).await;
                vastrum_abi::__private::vastrum_native_lib::deployers::deploy::poll_until_site_id_deployed(site_id).await;
                Self::new(site_id)
            }
        }
    } else {
        quote! {

            /// Deploys contract, will only return when contract has actually been deployed and polls for inclusion
            /// Can take some time for this function to return because of this
            #[cfg(not(target_arch = "wasm32"))]
            pub async fn deploy(wasm_path: &str) -> Self {
                let site_id = vastrum_abi::__private::vastrum_native_lib::deployers::deploy::deploy_module(
                    wasm_path, vec![]
                ).await;
                vastrum_abi::__private::vastrum_native_lib::deployers::deploy::poll_until_site_id_deployed(site_id).await;
                Self::new(site_id)
            }
        }
    }
}

/// Generates async method implementations
fn generate_async_methods_from_info(methods: &[&MethodInfo]) -> TokenStream {
    let mut impls = Vec::new();

    for method in methods {
        let method_name = format_ident!("{}", method.name);
        let sel = calculate_function_selector(&method.name);

        let mut signature_params = Vec::new();
        for p in &method.params {
            let name = format_ident!("{}", p.name);
            if matches!(p.ty, AbiType::String) {
                signature_params.push(quote! { #name: impl Into<String> });
            } else {
                let ty = p.ty.to_tokens();
                signature_params.push(quote! { #name: #ty });
            }
        }

        let mut param_names = Vec::new();
        for p in &method.params {
            param_names.push(format_ident!("{}", p.name));
        }

        let calldata_build = if method.params.is_empty() {
            quote! { let calldata = vec![#(#sel),*]; }
        } else {
            // Convert String params using .into()
            let mut conversions = Vec::new();
            for p in &method.params {
                let name = format_ident!("{}", p.name);
                if matches!(p.ty, AbiType::String) {
                    conversions.push(quote! { let #name: String = #name.into(); });
                }
            }

            quote! {
                #(#conversions)*
                let mut calldata = vec![#(#sel),*];
                calldata.extend(vastrum_abi::__private::borsh::to_vec(&(#(#param_names),*)).unwrap());
            }
        };

        let call_expr = if method.requires_auth {
            quote! { self.client.make_authenticated_call(calldata).await }
        } else {
            quote! { self.client.make_call(calldata).await }
        };

        let method_impl = quote! {
            pub async fn #method_name(&self, #(#signature_params),*) -> vastrum_abi::__private::vastrum_rpc_client::SentTx {
                #calldata_build
                #call_expr
            }
        };

        impls.push(method_impl);
    }

    quote! { #(#impls)* }
}

/// Generates state struct and reader using typed field info
fn generate_field_getters_from_info(struct_name: &str, fields: &[FieldInfo]) -> TokenStream {
    let native_struct_name = format_ident!("Native{}", struct_name);

    let mut native_fields = Vec::new();
    let mut field_types = Vec::new();
    let mut field_names = Vec::new();
    let mut field_constructions = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let field_name = format_ident!("{}", field.name);
        let tuple_field = format_ident!("f{}", i);

        // Native struct field using typed native tokens
        let native_type = field.ty.to_native_tokens();
        native_fields.push(quote! { pub #field_name: #native_type });

        // Tuple field name
        field_names.push(tuple_field.clone());

        // Deserialization type and construction based on AbiType
        match &field.ty {
            AbiType::KvMap { key, value } => {
                field_types.push(quote! { u64 });
                let k = key.to_tokens();
                let v = value.to_tokens();
                field_constructions.push(quote! {
                    #field_name: vastrum_abi::__private::vastrum_native_types::KvMap::<#k, #v>::new(#tuple_field, client.clone())
                });
            }
            AbiType::KvVec { elem } => {
                field_types.push(quote! { u64 });
                let t = elem.to_tokens();
                field_constructions.push(quote! {
                    #field_name: vastrum_abi::__private::vastrum_native_types::KvVec::<#t>::new(#tuple_field, client.clone())
                });
            }
            AbiType::KvBTree { key, value } => {
                field_types.push(quote! { u64 });
                let k = key.to_tokens();
                let v = value.to_tokens();
                field_constructions.push(quote! {
                    #field_name: vastrum_abi::__private::vastrum_native_types::KvBTree::<#k, #v>::new(#tuple_field, client.clone())
                });
            }
            AbiType::KvVecBTree { sort, value } => {
                field_types.push(quote! { (u64, u64) });
                let s = sort.to_tokens();
                let v = value.to_tokens();
                field_constructions.push(quote! {
                    #field_name: vastrum_abi::__private::vastrum_native_types::KvVecBTree::<#s, #v>::new(#tuple_field.0, #tuple_field.1, client.clone())
                });
            }
            _ => {
                // Direct types (primitives, user-defined, etc.)
                let ty = field.ty.to_native_tokens();
                field_types.push(ty);
                field_constructions.push(quote! { #field_name: #tuple_field });
            }
        }
    }

    quote! {
        #[derive(Debug)]
        pub struct #native_struct_name {
            #(#native_fields),*
        }

        #[allow(dead_code)]
        pub struct StateReader {
            client: Arc<vastrum_abi::__private::vastrum_rpc_client::RpcClient>,
        }

        impl StateReader {
            pub fn new(site_id: vastrum_abi::__private::vastrum_shared_types::crypto::sha256::Sha256Digest) -> Self {
                Self { client: Arc::new(vastrum_abi::__private::vastrum_rpc_client::RpcClient::new(site_id)) }
            }

            pub fn from_client(client: Arc<vastrum_abi::__private::vastrum_rpc_client::RpcClient>) -> Self {
                Self { client }
            }

            #[allow(unused_parens)]
            pub async fn state(&self) -> #native_struct_name {
                let bytes = self.client.get_key_value("__state".to_string()).await.unwrap();
                let client = self.client.clone();
                let (#(#field_names),*): (#(#field_types),*) = vastrum_abi::__private::vastrum_native_types::with_deser_client(&client, || {
                    vastrum_abi::__private::borsh::from_slice(&bytes).unwrap()
                });

                #native_struct_name {
                    #(#field_constructions),*
                }
            }
        }
    }
}

/// Pipe source code through rustfmt for formatting.
fn try_rustfmt(source: &str) -> Option<std::string::String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .arg("--edition=2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    child.stdin.take()?.write_all(source.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;

    if output.status.success() { std::string::String::from_utf8(output.stdout).ok() } else { None }
}

pub fn generate(contract_src_dir: &std::path::Path) -> Option<std::string::String> {
    let contract_def = parse_contract_source(contract_src_dir).ok()?;
    let contract_info = contract_def_to_info(&contract_def);
    let code = generate_abi_code_from_info(&contract_info);
    let source = code.to_string();
    let formatted = try_rustfmt(&source).unwrap_or(source);
    Some(format!("// This file is @generated by abi-codegen. Do not edit.\n\n{formatted}"))
}
