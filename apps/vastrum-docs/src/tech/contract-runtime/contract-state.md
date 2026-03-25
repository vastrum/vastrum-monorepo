# Contract State

This is how the contract_state looks like

```rust
#[contract_state]
struct Contract {
    repo_store: KvMap<String, GitRepository>,
    all_repos: KvVecBTree<u64, GitRepository>,
    forks_store: KvMap<ForksKey, Vec<String>>,
    git_object_store: KvMap<Sha1Hash, Vec<u8>>,
}
#[contract_methods]
impl Contract {
    #[authenticated]
    pub fn create_pull_request(
        &mut self,
        to_repo: String,
        merging_repo: String,
        title: String,
        description: String,
    )
```



The contract macro generates function handler for each external function.

For each function call, the contract state (struct Contract {}) is loaded from kv storage and then after execution is finished the current state is written to kv storage.

Generated contract macro code.
```rust


impl Contract {
    fn __load() -> Self {
        let bytes = runtime::kv_get("__state");
        borsh::from_slice(&bytes).unwrap()
    }
    fn __save(&self) {
        runtime::kv_insert("__state", &borsh::to_vec(self).unwrap());
    }
}

fn __external_handler_create_pull_request(params_bytes: &[u8]) {
    let mut contract = Contract::__load();
    let params: __ExternalCreatePullRequestParams = borsh::from_slice(params_bytes)
        .unwrap();
    contract
        .create_pull_request(
            params.to_repo,
            params.merging_repo,
            params.title,
            params.description,
        );
    contract.__save();
}


```


Full call stack from makecall external hostbinding

```rust
fn dispatch(input: &[u8]) {
    let selector = &input[0..8];
    let params = &input[8..];
    match selector {
        [190u8, 205u8, 101u8, 177u8, 113u8, 70u8, 25u8, 119u8] => {
            __external_handler_create_pull_request(params)
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn makecall(ptr: *const u8, len: u32) {
    __setup_panic_hook();
    let input = unsafe { core::slice::from_raw_parts(ptr, len as usize) };
    dispatch(input);
}
```
