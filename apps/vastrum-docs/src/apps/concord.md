# Concord

<!-- concord -->[x647757zpbejyzxcw7ruqcju32otdmi7vphrg36vhhzglkjccaqq.vastrum.net](https://x647757zpbejyzxcw7ruqcju32otdmi7vphrg36vhhzglkjccaqq.vastrum.net)

Concord is a heavily vibecoded attempt at creating a credible decentralized alternative to Discord.



I believe it is possible to achieve feature parity with Discord, some of the remaining features needed for that
- Revokable invite links
    - Limited amount of joined users per invite link
- Good application level anti spam tooling
- P2P voice chat without leaking IP (would have to go outside blockchain for this)
    -   Video probably also
    -   Streaming desktop also
- Good moderation tools
- Good account recovery ux (ie email recovery)
- Rich embeddings in messages (pictures, videos, pdfs)
- Native desktop/mobile application
    

## How it works

This is parts of the contract for Concord.


```rust
#[contract_state]
struct Contract {
    servers: KvVec<Server>,
    user_profiles: KvMap<Ed25519PublicKey, UserProfile>,
    dm_conversations: KvMap<DmKey, DmConversation>,
}

#[contract_methods]
impl Contract {
    #[authenticated]
    pub fn send_message(&mut self, server_id: u64, channel_id: u64, content: String) {
        let sender = message_sender();
        if content.len() > MAX_MESSAGE_LEN {
            return;
        }
        let server = self.servers.get(server_id).unwrap();

        let channel = server.get_channel(channel_id);

        let now = block_time();
        let msg = ChannelMessage {
            id: channel.messages.next_id(),
            content,
            author: sender,
            timestamp: now,
        };
        channel.messages.push(now, msg);
    }
```

Concord is structured like Discord with servers and channels within servers.

All of the metadata is in plaintext, however chat message contents are all encrypted.

```rust
#[contract_type]
struct Server {
    id: u64,
    name: String,
    owner: Ed25519PublicKey,
    members: KvBTree<Ed25519PublicKey, u8>,
    channels: Vec<Channel>,
    next_channel_id: u64,
}
```

When you create a server, you generate a new private key. All communication within that server will be encrypted using that private key.

To invite other users to the server, you share an invite link that contains the private key to the server.

When a user joins a server, he writes that key to their userprofile onchain under encrypted_server_keys.

This is so access is persisted, if the user comes back on another computer he can sync his servers from the smart contract.

This encryption key is then used by all participants in the server to encrypt their sent messages

```rust
#[contract_type]
struct UserProfile {
    dm_activity: KvVecBTree<u64, DmKey>,
    encrypted_server_keys: Vec<(u64, [u8; 64])>,
    display_name: Option<String>,
}
```
Messages in channels are stored in a KvVecBtree which allows for efficient paginated queries even if the BTree has many entries.

See [KV Structure docs](../tech/contract-runtime/kv-structure.md) for more info

```rust
#[contract_type]
struct Channel {
    id: u64,
    name: String,
    messages: KvVecBTree<u64, ChannelMessage>,
}
```


Concord has a lot of half baked vibecoded features like member lists and similar

Anybody can "join" a server there is no onchain checks, the only hard checks is having access to the private key so you can actually decrypt and see the messages in the server

The notification systems is also pretty bad and hacked so that users get notifications from messages. This adds quite a bit of complexity to the contract.

There is most probably better ways of doing all of these, but it is mostly a proof of concept. The primary goal was to emulate as many of Discord features as possible, even when it did not make sense to do so in a smart contract context.






## Specific implementation details


Concord follows standard app pattern
- Deploy > Rust crate that handles building frontend and contract and deploying contract + frontend to blockchain
- ABI > Macro generated code for automatically creating bindings between frontend and contract
- Contract > Rust WASM contract backend
- Frontend > react + Rust WASM frontend


<!-- gitter -->[Concord on Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/apps/concord)
