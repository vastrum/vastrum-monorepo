use sha2::{Digest, Sha256};

/// Computes the first 8 bytes of SHA256 hash of the function name.
pub fn calculate_function_selector(name: &str) -> [u8; 8] {
    let hash = Sha256::digest(name.as_bytes());
    let mut function_selector = [0u8; 8];
    function_selector.copy_from_slice(&hash[..8]);
    return function_selector;
}
