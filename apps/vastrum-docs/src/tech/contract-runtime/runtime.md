# Runtime


The contract runtime exposes these host functions

message_sender, the account which sent the transactions, intended for #[authenticated] functions who need to authenticate actions based on sender.


register_static_route, register html for website at a route, "" is the catchall route where all page requests fallback to.


kv_insert and kv_get, for reading and writing to the sitekv database backed by RocksDB

```rust
impl HostRuntime for HostState {
    fn message_sender(&self) -> Vec<u8> {
        let sender: Ed25519PublicKey = self.message_sender.into();

        let response = GetMessageSenderResponse { sender };
        return response.encode();
    }

    fn block_time(&self) -> u64 {
        return self.block_timestamp;
    }

    fn register_static_route(&mut self, args: &[u8]) {
        let Ok(RegisterStaticRouteCall { route, brotli_html_content }) = borsh::from_slice(args)
        else {
            tracing::warn!("failed to decode RegisterStaticRouteCall");
            return;
        };
        let page = Page { site_id: self.site_id, path: route, brotli_html_content };
        self.db.write_page(page);
    }

    fn kv_insert(&mut self, args: &[u8]) {
        let Ok(KeyValueInsertCall { key, value }) = borsh::from_slice(args) else {
            tracing::warn!("failed to decode KeyValueInsert");
            return;
        };
        if value.is_empty() {
            self.db.delete_kv(&key, self.site_id);
        } else {
            self.db.write_kv(&key, value, self.site_id);
        }
    }

    fn kv_get(&self, args: &[u8]) -> Vec<u8> {
        let Ok(KeyValueReadCall { key }) = borsh::from_slice(args) else {
            tracing::warn!("failed to decode KeyValueRead");
            return KeyValueReadResponse { value: vec![] }.encode();
        };
        let value = self.db.read_kv(&key, self.site_id).unwrap_or(vec![]);
        let response = KeyValueReadResponse { value };
        return response.encode();
    }

    fn log(&mut self, args: &[u8]) {
        let Ok(LogCall { message }) = borsh::from_slice(args) else {
            tracing::warn!("failed to decode Log");
            return;
        };
        tracing::info!(site_id = ?self.site_id, "{}", message);
    }
}
```


## #[authenticated] vs normal

For authenticated calls the client uses a persistent identity, for non #[authenticated] calls a one off account is randomly generated for each transaction sent to avoid linking the transaction to a persistent identity when it is not needed.

