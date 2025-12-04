use shared_types::crypto::ed25519;

pub fn insecure_generate_new_static_identity(seed: u64) -> Keystore {
    //TODO: unknown relevant footguns for rng

    //used for transactions, most sensitive
    let private_key = ed25519::PrivateKey::from_seed(seed);

    //used for signing, less sensitive
    let signing_key = ed25519::PrivateKey::from_seed(seed);

    //used for signing p2p identity, maybe does not need to do ed25519
    let p2p_key = ed25519::PrivateKey::from_seed(seed);

    let keystore =
        Keystore { private_key: private_key, signing_key: signing_key, p2p_key: p2p_key };
    return keystore;
}

#[derive(Clone, Debug)]
pub struct Keystore {
    pub private_key: ed25519::PrivateKey,
    pub signing_key: ed25519::PrivateKey,
    pub p2p_key: ed25519::PrivateKey,
}
