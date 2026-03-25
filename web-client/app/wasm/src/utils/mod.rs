pub mod error;
pub mod site_id;

pub fn get_random_u64() -> u64 {
    let mut bytes = [0u8; 8];
    let window = web_sys::window().unwrap();
    let crypto = window.crypto().unwrap();
    crypto.get_random_values_with_u8_array(&mut bytes).unwrap();

    u64::from_le_bytes(bytes)
}
