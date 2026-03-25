pub use vastrum_runtime_shared::calculate_function_selector;

/// Converts snake_case to PascalCase
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    for part in s.split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            for c in first.to_uppercase() {
                result.push(c);
            }
            result.extend(chars);
        }
    }
    return result;
}
