# Developing On Vastrum

To develop on Vastrum you need to install Rust and Node first, then you will need to install the Vastrum node to locally test your website. 

There are two scaffolds available to start a new project, these scaffolds will setup a basic frontend and backend along with a deployment script.

## 1. Install dependencies


Install [Rust](https://rustup.rs)

Install [Node.js](https://nodejs.org)

## 2. Install Vastrum to locally run a Vastrum node to deploy websites on
```bash
curl -sSf https://raw.githubusercontent.com/vastrum/vastrum-monorepo/HEAD/tooling/cli/install.sh | sh
```
```bash
rustup target add wasm32-unknown-unknown
```
```bash
cargo install wasm-pack
```

## 3. Scaffold a new site

### Regular websites such as forums
    vastrum-cli init <name> --template site                                       

### DeFi applications
    vastrum-cli init <name> --template eth_dapp   