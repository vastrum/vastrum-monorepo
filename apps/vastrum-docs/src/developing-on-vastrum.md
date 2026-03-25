# Developing On Vastrum

If you want to develop a website on Vastrum there are two scaffolds available.


## Install CLI and dependencies first

Install [Rust](https://rustup.rs)

Install [Node.js](https://nodejs.org)

```bash
curl -sSf https://raw.githubusercontent.com/vastrum/vastrum-monorepo/HEAD/tooling/cli/install.sh | sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```
    
## Site Scaffold

This scaffolds

-   React frontend + embedded Rust WASM frontend to communicate with backend contract
-   Rust WASM smart contract backend
-   ABI crate to generate bindings for smart contract
-   Deploy crate to deploy site

Used for developing websites.

    vastrum-cli init <name> --template site                                       


## ETH dApp scaffold

This scaffold is intended for developing DeFi frontends.
-   React frontend + viem
-   Rust smart contract + deploy + ABI crate 
    -   Only used in order to deploy the frontend, you only have to care about the react frontend

The EIP-1193 provider can be used with other libraries than viem also.

    vastrum-cli init <name> --template eth_dapp   




## Testing

Currently cannot deploy anything to protonet, however can locally deploy and test applications.

    vastrum-cli run-dev

## Vibe coding

If you want to try vibe coding something or you want to use LLMs to help with your coding, download the vastrum-monorepo and try to modify an app. The LLM works much better when it can directly inspect all library code such as vastrum-node, native-lib and runtime-lib.



----
*All assets needed by the frontend need to embedded and inlined into a single HTML file, this breaks a lot of common frontend development workflows. You will definitely have problems with this but i am working on a better solution. For now, most of the time you can vibecode a solution for inlining fonts, pictures and other static assets relatively easy.*