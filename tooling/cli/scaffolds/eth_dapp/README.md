This is a scaffolded Vastrum app. This is basically just a regular Viem app except it is hosted on Vastrum and uses Helios for the RPC queries.

To test locally

```bash
vastrum-cli run-dev
```



Vastrum docs

https://docs.vastrum.net


## Project Structure

- `contract/` - Smart contract (Rust, compiled to WASM)
- `abi/` - Auto-generated ABI bindings (do not edit manually)
- `deploy/` - Deployment script
- `frontend/` - React + Vite web UI
- `frontend/wasm/` - WASM module for frontend logic

