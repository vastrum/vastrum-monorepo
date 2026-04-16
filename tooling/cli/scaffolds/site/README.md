This is a scaffolded Vastrum app. The app is basically just an interface for setting keyvalue values and reading them.

## To run locally

```bash
vastrum-cli run-dev
```

This starts a local node, opens the browser, and deploys your contract. The site will load automatically once deployment completes.



If you want to change this app there are two folders that are interesting.

The /contract/ folder contains the contract Rust crate, this is the backend for the project which is executed on the blockchain.

The /frontend/ folder contains the react frontend, this frontend uses a Rust WASM component (/frontend/wasm/) to read data from the backend and write transactions to it.

The vastrum-monorepo also has many example apps which can be used as reference for how to develop apps.

Gitter - Github clone

https://gitter.vastrum.net/repo/vastrum/tree/apps/gitter/contract/src/lib.rs


Vastrum docs

https://docs.vastrum.net


## Project Structure

- `contract/` - Smart contract (Rust, compiled to WASM)
- `abi/` - Auto-generated ABI bindings (do not edit manually)
- `deploy/` - Deployment script
- `frontend/` - React + Vite web UI
- `frontend/wasm/` - WASM module for frontend logic

