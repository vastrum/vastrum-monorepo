# Architecture


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
