# Introduction

Vastrum is an experimental protocol for hosting decentralized websites and services.

Current prototype apps built and hosted on Vastrum

-   [Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net)      -   Decentralized alternative to Github - [vastrum-monorepo hosted on Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum)
-   [Swapper](https://uy25lxmolovvfrw6dckfnh3qi4pm2ah5lutj6k2ljqburlbjegta.vastrum.net)     -   Decentralized Uniswap V2 frontend using Helios light client for RPC reads
-   [Chatter](https://vaaxjx64bdwibfoc6nwxkelica3lfppxwilmws4s4bskfknkr7ra.vastrum.net)     -   No-metadata private messaging
-   [Concord](https://x647757zpbejyzxcw7ruqcju32otdmi7vphrg36vhhzglkjccaqq.vastrum.net)     -   Decentralized Discord
-   [Concourse](https://xv6edwxtsjtlujz2z7hgbkkshwx2im6bbvx3nxzyczumsnhwddrq.vastrum.net)   -   Decentralized Discourse
-   [Letterer](https://yozq5azfm26qi3vceclwz57fg2727yhqi6ccha5khhnp2uepqj7a.vastrum.net)    -   Decentralized Google Docs
-   [Mapper](https://lzdtxcpp6ivwje55o74dugj7f4vie6qzrsp6kybqyi7ofo3yt75q.vastrum.net)      -   Decentralized Google Maps
-   [Blocker](https://d66m4cniuqbgkeuetyvcbkfqfutt3qd3hdxdby2tlqifbt3otctq.vastrum.net)     -   Blockchain explorer for Vastrum, hosted on Vastrum
-   Docs - the current website you are on


## FAQ

### How is Vastrum different from IPFS and other similar decentralized website hosting services?

**Interactivity**
- Vastrum executes the websites backend onchain which allows for interactivity, IPFS is mostly for static content
- This means you can implement forums, chat applications, decentralized Github fully onchain, without any external centralized API dependencies

**Hosting model**
- IPFS is a content addressed routing network, it will not host your files for you natively. You need to pay for a centralized pinning service like Pinata or self host it.
- Vastrum is a full stack decentralized hosting services which will host your website and ensure you can access it forever once it is hosted.

**Trustless access**
- All Vastrum clients are light-clients and verifies the retrieved website content hash against consensus using a JMT state hash. Most previous solutions just blindly trust the retrieved data from the gateway.

**Connection**
- Most solutions use centralized HTTP gateways, Vastrum uses WebRTC to directly connect to the RPC node and fetch the data

**Isolation model**
- Most decentralized hosting solutions give full access to the website, it can make arbitrary requests and internet connections. 
- Vastrum sandboxes all websites inside an IFrame which prevents all internet connections. Instead a limited isolated runtime API is provided to websites to read data from the Vastrum network and the Ethereum network.

**Free write transactions**
- You do not have to pay to make a post. You also do not need to verify a telephone number or create an account.
- This works by having the user solve a cryptographic puzzle to burn electricity as a [proof of work of value spent](tech/feasibility-estimates/dos-resistance.md). This means sending each transactions spends some economic value. This helps prevent spam DOS attacks.
- I estimate it will cost [roughly 10-1000 USD per hour](tech/feasibility-estimates/dos-resistance.md#simple-model-of-a-dos) in electricity/compute costs for an attacker to DOS the network.
- While this is low, if you want to DOS the network for a long time it quickly becomes a non trivial cost.
- There are [many potential ways to improve this](tech/feasibility-estimates/dos-resistance.md#further-improvements-to-dos-resistance).

**Comprehensive Development Tooling**
- Easy to deploy.
- Full local dev CLI.
- ABI contract binding macros for the frontend.
- Regular React tech for frontend (HTML/CSS/JS)

**Bring your own Web2 domain**

Potentially possible via [threshold TLS](roadmap/treshold-tls.md), which would eliminate the frontend centralization risk of the current vastrum.net gateway by giving the initial web-client load economic security on your own domain.

### What has been built?

- Fully custom blockchain implementation with Simplex consensus, very compact with just 6K lines of Rust.
    - Current production deployment has 8 validator nodes/servers
    - [4 blocks per second currently](https://d66m4cniuqbgkeuetyvcbkfqfutt3qd3hdxdby2tlqifbt3otctq.vastrum.net) (will be lower on heavier block load)
    - [Comprehensive deterministic integration testing of consensus](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/vastrum-node/tests/sim_consensus.rs)
- Fully custom Rust WASM based smart contract runtime
    - 50K TPS execution benchmark (increment_counter() single cached contract)
- Rust based web-client that uses WebRTC-direct to directly connect to a RPC node, runs a Vastrum light-client and verifies all queries against consensus.
    - Built in wallet for the web-client
- Helios integration inside frontend runtime to natively support trustless ETH RPC queries for DeFi frontends (No external centralized ETH RPC provider needed)
- Comprehensive developer tooling (ABI macro for contract bindings, contract runtime library for keyvalue datastructures like KvMap)
- 8 example apps (Github clone, decentralized Uniswap V2 frontend, forum/chat apps)

### How ready is it for production usage?

Vastrum is currently in heavy developement and i regularly restart the blockchain to test new functionality that is not backwards compatible.

However all the apps works currently, you can push your Git repo to Gitter for example.

I will probably stabilize it soon and guarantee data persistency. If you have a specific usecase i can defintively do this sooner.


## Summary 

All Vastrum websites are available through the vastrum.net web-client gateway using any web browser.

There is no gas, only POW as anti DOS, you can directly interact with any website like any regular Web2 website. 

You do not have to install a wallet, the private key is stored by the vastrum.net web-client in local storage.


## Tech stack

-   The frontend for websites are written using normal HTML, CSS, JS

-   All websites are SPA apps (React only currently)

-   The website is sandboxed inside an iframe in the web-client

-   The web-client fetches dApp website pages from the RPC node and renders it inside the iframe

-   The backend is a Rust WASM smart contract executed onchain

-   Backend state is held in a per site keyvalue database 

-   The dApp website directly reads from the keyvalue database 

-   All page reads and keyvalue reads are proven and verified against current consensus state hash using Jellyfish Merkle Trees

-   Clients connect directly to RPC nodes using webRTC-direct

-   Very lightweight, 6k lines of Rust for vastrum-node, 30k lines of Rust for all apps+runtime+tooling+tests+webrtc-direct+web-client+vastrum-node


This is a pretty radical architecture which presents many problems. If you are interested in the proposed solutions to these problems check [General Architecture Decisions](tech/general-architecture-decisions.md)


<br>
<br>

```
+---------------+                        +------------------------------------+
|               |                        |     Web Browser > vastrum.net      |
|   RPC Node    |       WebRTC           |                                    |
|               | ---------------------->|  +------------------------------+  |
|   +-------+   |                        |  |  web-client                  |  |
|   |  RPC  |   |<---------------------- |  |        ^                     |  |
|   +-------+   |                        |  |        | postMessage()       |  |
|               |                        |  |        |                     |  |
|   +-------+   |                        |  |        v                     |  |
|   |  P2P  |   |                        |  |  +------------------------+  |  |
|   +---+---+   |                        |  |  |  <iframe>              |  |  |
|       |       |                        |  |  |                        |  |  |
+-------+-------+                        |  |  |   [ dApp frontend ]    |  |  |
        |                                |  |  |                        |  |  |
        |  TCP                           |  |  +------------------------+  |  |
        |                                |  |                              |  |
        |                                |  +------------------------------+  |
        |                                +------------------------------------+
        |
        |
   +----+--------------------------------------+
   |                                           |
+--+-----------+   +-------------+   +-------------+
| Validator 1  |---| Validator 2 |---| Validator 3 |
+------+-------+   +------+------+   +------+------+
       |                  |                  |
+------+-------+   +------+------+   +------+------+
| Validator 4  |---| Validator 5 |---| Validator.. |
+--------------+   +-------------+   +-------------+
```
### Example flow for loading Swapper - a decentralized Uniswap V2 frontend
1. User goes to {SITE_ID}.vastrum.net
2. The web-client frontend is loaded.
3. The web-client connects to a random RPC node using webRTC-direct
4. Web-client requests get_page({SITE_ID})
5. RPC node generates response + merkle tree proof + most recent consensus certificate
6. Web-client receives response, verifies merkle tree proof + most recent consensus certificate
7. Web-client loads the page/swapper frontend into a sandboxed iframe and it is rendered to the browser
8. Swapper now has code execution inside the sandboxed iframe,
9. Swapper sends a postMessage() request to the parent iframe host containing an ETH RPC request.
10. The web client starts the web-worker for Helios, proxies all RPC requests by Helios to the Vastrum RPC node using webRTC direct
11. Helios verifies the responses and returns the data to web-client
12. Web-client uses postMessage() to send back the response for the RPC request to Swapper

---
-   The random RPC node is selected from a bootstrap list embedded in web-client (In future could be read from onchain registry)
-   The genesis validator set is also embedded in web-client and is used by the web-client to verify state proofs (currently static validator set)
-   Swapper never has any internet connection or access to anything outside sandboxed iframe, web-client handles all network communication

[github.com/vastrum/vastrum-monorepo](https://github.com/vastrum/vastrum-monorepo)
