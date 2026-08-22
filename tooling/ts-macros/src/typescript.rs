use syn::{Data, DeriveInput, Fields, Type};

pub fn declaration(input: &DeriveInput) -> Result<String, syn::Error> {
    let name = input.ident.to_string();
    match &input.data {
        Data::Struct(data) => struct_declaration(&name, &data.fields),
        Data::Enum(data) => enum_declaration(&name, data),
        Data::Union(_) => {
            Err(syn::Error::new_spanned(&input.ident, "unions cannot be exported to TypeScript"))
        }
    }
}

fn struct_declaration(name: &str, fields: &Fields) -> Result<String, syn::Error> {
    let Fields::Named(named) = fields else {
        return Err(syn::Error::new_spanned(
            fields,
            "only structs with named fields can be exported to TypeScript",
        ));
    };
    let mut declaration = format!("export interface {name} {{\n");
    for field in &named.named {
        let field_name = field.ident.as_ref().expect("named field has an identifier");
        declaration.push_str(&format!("    {}: {};\n", field_name, type_script_type(&field.ty)?));
    }
    declaration.push_str("}");
    Ok(declaration)
}

fn enum_declaration(name: &str, data: &syn::DataEnum) -> Result<String, syn::Error> {
    let mut variants = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "only enums whose variants all carry no data can be exported to TypeScript",
            ));
        }
        variants.push(format!("\"{}\"", variant.ident));
    }
    Ok(format!("export type {name} = {};", variants.join(" | ")))
}

/// Map a Rust type onto its TypeScript spelling.
pub fn type_script_type(ty: &Type) -> Result<String, syn::Error> {
    match ty {
        Type::Path(path) => path_type(ty, path),
        Type::Array(array) => Ok(format!("{}[]", parenthesize(&type_script_type(&array.elem)?))),
        Type::Reference(reference) => type_script_type(&reference.elem),
        other => Err(syn::Error::new_spanned(other, "unsupported type in a TypeScript export")),
    }
}

fn path_type(ty: &Type, path: &syn::TypePath) -> Result<String, syn::Error> {
    let segment = match path.path.segments.last() {
        Some(segment) => segment,
        None => return Err(syn::Error::new_spanned(ty, "empty type path")),
    };
    let name = segment.ident.to_string();
    let arguments = generic_arguments(segment);

    match name.as_str() {
        "String" | "str" => Ok("string".to_string()),
        "bool" => Ok("boolean".to_string()),
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128"
        | "isize" | "f32" | "f64" => Ok("number".to_string()),
        "Option" => {
            let inner = single_argument(ty, &arguments, "Option")?;
            Ok(format!("{inner} | null"))
        }
        "Vec" | "VecDeque" | "HashSet" | "BTreeSet" => {
            let inner = single_argument(ty, &arguments, &name)?;
            Ok(format!("{}[]", parenthesize(&inner)))
        }
        "HashMap" | "BTreeMap" => {
            if arguments.len() != 2 {
                return Err(syn::Error::new_spanned(ty, "map types need a key and a value type"));
            }
            Ok(format!("Record<{}, {}>", arguments[0], arguments[1]))
        }
        "Box" | "Rc" | "Arc" | "Cow" => single_argument(ty, &arguments, &name),
        _ => Ok(name),
    }
}

fn generic_arguments(segment: &syn::PathSegment) -> Vec<String> {
    let syn::PathArguments::AngleBracketed(bracketed) = &segment.arguments else {
        return Vec::new();
    };
    let mut mapped = Vec::new();
    for argument in &bracketed.args {
        if let syn::GenericArgument::Type(inner) = argument {
            match type_script_type(inner) {
                Ok(rendered) => mapped.push(rendered),
                Err(_) => return Vec::new(),
            }
        }
    }
    mapped
}

fn single_argument(ty: &Type, arguments: &[String], name: &str) -> Result<String, syn::Error> {
    match arguments.first() {
        Some(argument) => Ok(argument.clone()),
        None => Err(syn::Error::new_spanned(ty, format!("{name} needs a type argument"))),
    }
}

fn parenthesize(rendered: &str) -> String {
    if rendered.contains(" | ") { format!("({rendered})") } else { rendered.to_string() }
}
