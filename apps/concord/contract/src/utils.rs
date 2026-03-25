use crate::DmKey;
use vastrum_runtime_lib::Ed25519PublicKey;

pub fn dm_key(a: Ed25519PublicKey, b: Ed25519PublicKey) -> DmKey {
    if a.bytes < b.bytes { DmKey { user_a: a, user_b: b } } else { DmKey { user_a: b, user_b: a } }
}
