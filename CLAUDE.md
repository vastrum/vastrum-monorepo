# Vastrum Monorepo

Decentralized website hosting protocol. Consensus node runs WASM smart contracts; browser connects via WebRTC DataChannel.

## Build & Test Commands

**Local development (starts localnet + frontend + deploys contracts):**
```bash
cd apps/<app-name> && cargo run -p vastrum-cli -- run-dev
```
Must be run from a dApp directory (one with a `deploy/` subdirectory).


**Quick validation:**
```bash
cargo check
```

**Tests:**
```bash
cargo test -p vastrum-node
cargo test -p runtime-tests          # contract runtime (state, KV, auth, domains, pages)
cargo test -p vastrum-git-lib         # gitter git ops (merge, diff, push, directory explorer)
cargo test -p vastrum-webrtc-direct-server   # WebRTC echo round-trip integration test
cargo test -p chatter-wasm           # chat state management
cargo test -p vastrum-shared-types           # crypto encryption
cargo test -p web-client-integration-tests  # iframe RPC and Helios Ethereum RPC
```


**Madsim simulation tests:**
```bash
RUSTFLAGS="--cfg madsim" cargo test -p vastrum-node --features madsim_compliant --test sim_consensus
```

**Benchmarks:**
```bash
cd runtime/runtime-benchmark && cargo run --release  # runtime performance benchmarks
```

## Architecture

- **Consensus:** Simplex protocol (leader proposes → justify votes → finalize/skip). 2/3 stake threshold.
- **Transactions:** Proof-of-Work based (no gas/fees). PoW hash must be below threshold, expires after 300 blocks.
- **Contract runtime:** WASM executed via Wasmtime. 256MB memory limit per execution.
- **Browser transport:** WebRTC DataChannel direct to node (no HTTP relay). Sites render in sandboxed iframes.
- **Storage:** Contracts use KV host bindings (`kv_insert`/`kv_get`). Higher-level: KvMap, KvVec, KvBTree, KvVecBTree.

## Project Structure

- **`vastrum-node/`** - consensus node (P2P, consensus, DB, RPC, contract execution)
- **`shared-types/`** - crypto primitives (Ed25519, X25519, ChaCha20-Poly1305, SHA256), serialization, protocol types
- **`web-client/`** - browser client:
  - `app/` - React+Vite frontend + `wasm/` WASM client crate
  - `helios-worker/` - Helios Ethereum light client WASM worker
  - `integration-tests/` - iframe RPC and Helios integration tests
- **`webrtc-direct/`** - WebRTC DataChannel transport (server + client + SDP + chunking)
- **`apps/`** - dApps:
  - `gitter` - git repo hosting (repos, PRs, issues, discussions, SHA1-based object storage)
  - `chatter` - pub/sub messaging (inbox KvMap)
  - `concord` - Discord-like chat (servers, channels, DMs, role-based permissions)
  - `concourse` - forum (categories, posts, replies, bump ordering)
  - `blocker` - vastrum blockchain explorer
  - `swapper` - uniswap v2 frontend using helios rpc light client
  - `letterer` - E2E encrypted document editor
  - `mapper` - google maps like explorer
  - `vastrum-docs` - vastrum mdbook docs
  - Each dApp follows the pattern: `contract/` + `deploy/` + `abi/` + `frontend/wasm/`
- **`runtime/`** - contract execution framework:
  - `runtime-lib` - guest API (`message_sender()`, `block_time()`, `kv_insert/get`, `log`) and KV data structures
  - `contract-macros` - proc macros: `#[contract_state]`, `#[contract_methods]`, `#[authenticated]`, `#[constructor]`
  - `abi-codegen` - ABI code generation with SHA256-based function selectors (first 8 bytes)
  - `vastrum-abi` - re-export crate used by generated ABI code
  - `runtime-shared` - shared host/guest data structures
  - `native-types` - RPC-based read-only KV types for external clients (not in contract)
  - `vastrum-bindings` - raw WASM FFI interface (guest extern "C" + host trait)
  - `runtime-tests` - contract runtime tests and benchmarks
- **`tooling/`** - CLI and infrastructure:
  - `tooling/cli/` - CLI: `run-dev` (spawns localnet + frontend on :3000 + deploy script), `vastrum-git-clone`, `vastrum-git-push`
  - `tooling/rpc-client/` - RPC provider abstraction (native HTTP + WASM/iframe)
  - `tooling/native-lib/` - native runtime: HTTP client, tx polling, contract deployers
  - `tooling/react-vastrum-lib/` - React component library with WASM bindings (crate: `vastrum-react-lib`, npm: `@vastrum/react-lib`)
  - `tooling/vastrum-lib-rust-frontend/` - WASM Rust lib for iframe RPC, tx polling, Ethereum RPC forwarding (crate: `vastrum-frontend-lib`)
- **`vendored-helios/`** - local fork of Helios Ethereum light client (not a submodule - full source with local commits)

## Contract Conventions

Contracts are Rust compiled to WASM. Key macros:
- `#[contract_state]` on struct - generates `__load()`/`__save()` for KV persistence
- `#[contract_methods]` on impl - generates `makecall` entry point with selector-based dispatch
- `#[authenticated]` on method - requires Ed25519 signature; use `message_sender()` for caller identity
- `#[contract_type]` on struct/enum - derives BorshSerialize, BorshDeserialize, Clone, Default (enums get `#[default]` on first variant)
- `#[constructor]` on associated fn - generates `construct()` export, called once at deploy
- Function selectors: first 8 bytes of SHA256(method_name)
- All string inputs should have `MAX_*_LEN` validation constants

## Code Conventions

- **Serialization:** Borsh for all consensus/network types via `BorshExt` trait (`.encode()` / `Type::decode()`)
- **P2P:** actor model - no mutexes on I/O, separate reader/writer/heartbeat tasks
- **Madsim determinism:** `tokio::select! { biased; }`

## WebRTC-Direct

Browser ↔ node transport over WebRTC DataChannel. Crates: `webrtc-direct/protocol/`, `webrtc-direct/server/`, `webrtc-direct/client/`, `webrtc-direct/integration-tests/`.

**Tests:**
```bash
cargo test -p vastrum-webrtc-direct-protocol  # chunking unit tests
cargo test -p webrtc-direct-integration-tests    # integration tests
```
me.

## Post-Change Verification

After modifying code, always run the relevant tests before considering the task done:

- **`webrtc-direct/`** → `cargo test -p vastrum-webrtc-direct-protocol` + `cargo test -p webrtc-direct-integration-tests`
- **`vastrum-node/`** → `cargo test -p vastrum-node -- --test-threads=1`
- **`shared-types/`** → `cargo test -p vastrum-shared-types`
- **`runtime/`** or contracts → `cargo test -p runtime-tests`
- **`apps/gitter/`** → `cargo test -p vastrum-git-lib`
- **`apps/chatter/`** → `cargo test -p chatter-wasm`
- **Any change** → at minimum `cargo check`

## Notes

### The ABI crates are auto generated and does not need to be updated, they are auto synced by abi-gen

### This project is not yet live and any breaking change is acceptable, there is no need to maintain backwards compatability

## Prerequisites

- Rust, Node.js, clang (for rocksdb), wasm-pack
