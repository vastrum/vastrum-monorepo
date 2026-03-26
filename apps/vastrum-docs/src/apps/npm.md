# NPM


NPM and other package repositories could be implemented on Vastrum.


## The basic components needed for a package manager

-   Need to upload packages + verify creator signature
-   Need to be able to download packages + verify content hash and creator signature


These are very simple specs to Vastrum, the backend logic could be reduced to this contract.


```rust
#[contract_state]
struct Contract {
    package_owner: KvMap<String, Ed25519PublicKey>,
    package_content: KvMap<String, Vec<u8>>,
}

#[contract_methods]
impl Contract {
    pub fn register_package_name(&mut self, package_name: String) {
        let package_already_owned = self.package_owner.contains(package_name);
        if package_already_owned {
            return;
        }
        let owner = message_sender();
        self.package_owner.set(package_name, owner);
    }
    pub fn update_package(&mut self, package_name: String, bytes: Vec<u8>) {
        let is_owner = self.package_owner.get(package_name) == message_sender();
        if !is_owner {
            return;
        }
        package_content.set(package_name, bytes);
    }

}
```

Then in client just do this to download a package

```rust
async fn download_package(package_name: String) -> Vec<u8> {
    let client = ContractAbiClient::new(NPM_SITE_ID);
    let state = client.state().await;
    let bytes = state.package_content.get(package_name).await;
    return bytes;
}
```

To upload
```rust
async fn upload_package(package_name: String, package_bytes: Vec<u8>, private_key: Ed25519::PrivateKey) -> Vec<u8> {
    let client = ContractAbiClient::new(NPM_SITE_ID).with_private_key(private_key);
    client.update_package(package_name, package_bytes).await;
}
```


You would probably need some form of moderation and curation of malicious packages and typo squatting, could go fully uncontrolled, or could try to onboard current package repositories and expose certain moderation functionalities.

Would probably also want some way to track downloads or someway to determine the legitimacy of packages instead of assigning same trust score to all packages.

Would also probably want to split uploads up into multiple smaller transactions to allow for packages greater than current transaction size limit of 4 MB.