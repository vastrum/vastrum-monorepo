# Vastrum

Vastrum is an experimental protocol for hosting decentralized websites. See [documentation](https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net) for more info.


## Relevant Code

[Consensus Integration Tests](https://github.com/vastrum/vastrum-monorepo/blob/master/vastrum-node/tests/sim_consensus.rs)

[Example Contract For Gitter (Github clone)](https://github.com/vastrum/vastrum-monorepo/blob/master/apps/gitter/contract/src/lib.rs)

[Runtime Integration Tests](https://github.com/vastrum/vastrum-monorepo/blob/master/runtime/runtime-tests/tests/src/tests/kvmap.rs)

[Consensus State Machine](https://github.com/vastrum/vastrum-monorepo/blob/master/vastrum-node/src/consensus/validator_state_machine.rs)

[KvMap implementation](https://github.com/vastrum/vastrum-monorepo/blob/master/runtime/runtime-lib/src/kvmap.rs)

[Client side Git clone implementation for Gitter](https://github.com/vastrum/vastrum-monorepo/blob/master/apps/gitter/vastrum-git-lib/src/native/clone.rs)

## Setup

1. Install [Rust](https://rustup.rs)
2. Install [Node.js](https://nodejs.org)
3. Run:

```bash
sudo apt install clang build-essential liblz4-dev
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
cargo install mdbook
```

## Run a website locally

```bash
cd apps/gitter && cargo run -p vastrum-cli -- run-dev
```

## Deploy all websites locally

```bash
make deploy-all-localnet
```


## Scaffold project

### Install vastrum-cli

**Prebuilt binary:**
```bash
curl -sSf https://raw.githubusercontent.com/vastrum/vastrum-monorepo/HEAD/tooling/cli/install.sh | sh
```

**From source:**
```
make cli_install
```

### Scaffold options
```
vastrum-cli init <name> --template site                                       
vastrum-cli init <name> --template eth_dapp       
```


### Project structure

6K lines of Rust code for vastrum-node, 30K lines of Rust code for the whole monorepo (excluding Helios and jmt-main).

-   vastrum-node - Vastrum blockchain node
-   apps - Prototype apps
-   runtime - Libraries, tooling and tests for the smart contract runtime
-   shared-types - Shared internal library
-   web-client - Frontend served by vastrum.net
    -   app - Frontend
    -   helios-worker - Web worker that helios is hosted in
    -   integration tests
-   webrtc-direct - WebRTC-direct implementation
-   tooling - CLI, app libraries
-   vendored-helios - https://github.com/a16z/helios
-   vendored-jmt-main - https://github.com/penumbra-zone/jmt