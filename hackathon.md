Note: Vastrum requires WebRTC to be enabled to connect, if it is disabled the frontend cannot load the website data from the RPC node.

Vastrum is an experimental L1 for hosting decentralized websites and services.


https://docs.vastrum.net


## For this hackathon i implemented a decentralized DeFI frontend for Starknet using Vastrum.

Common problems with DeFi frontends
-   Disappears if developers stops hosting it, leaving abandoned onchain contracts no regular user can interact with
-   Need to trust RPC infrastructure, no way to verify the data displayed by the frontend is correct
-   Operational complexity, a contract can just be deployed but a frontend requires continuous maintenance in most cases.
-   Privacy, most DeFi frontend hosting stacks have many middlemen involved in hosting it, for example DDOS protection, CDN, analytics and more services. By reducing the amount of dependencies you increaes privacy and robustness.

I think to have fullstack decentralization the frontend also needs to be hosted onchain in some manner with the same decentralization guarantees that the contract has.

I have previously developed Swapper for Vastrum which was a Uniswap V2 frontend hosted on Vastrum using a Helios light client embedded inside the web browser to do verified RPC queries. 


The Starknet ecosystem has Beerus for verifying RPC queries in the browser environment, however it is abandoned. I forked Beerus and fixed the WASM compilation issues + API  incompatibilities in order to enable verified trustless Starknet RPC queries in the web browser environment.

The Beerus verified RPC calls were implemented using the standard light client method, locally execute the contract code, then get state proofs and verify storage reads. This allows you to verify the output of view functions. 

https://github.com/vastrum/Beeruser


I then integrated this with the Vastrum web-client to create starknet-frontend.vastrum.net. starknet-frontend is a frontend for the Ekubo DEX. The frontend verifies all RPC queries using state proofs verified by Beerus.

https://starknet-frontend.vastrum.net


The idea is that potentially any DeFI frontend could be hosted on Vastrum. I am interested in developing this further.

## Current problems. 

 -  Swapper uses WebRTC to directly connect to RPC node for Helios requests, Starknet uses HTTP requests.

- I could not find a good way to verify the state hash for Starknet, you could read the L1 state hash but that would be severely delayed. So in the current implementation you trust the RPC client for the state hash, however all requests are verified against that state hash. If starknet implements state hash verification for light clients then it would be a full stack light client.



Integration started from this commit

-   https://github.com/vastrum/vastrum-monorepo/commit/c2b51da09fa6f70b79e95784af98deb8401bdf03

Ended with this commit 

-   https://github.com/vastrum/vastrum-monorepo/commit/055f9cd2a190d9bd7bfe3b1ee5381e1e8984b820

I also implemented the Beerus fork

-   https://github.com/vastrum/Beeruser


### More info about Vastrum

Current prototype apps built and hosted on Vastrum

-   [Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net)      -   Decentralized Git forge - [vastrum-monorepo hosted on Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum)
-   [Swapper](https://uy25lxmolovvfrw6dckfnh3qi4pm2ah5lutj6k2ljqburlbjegta.vastrum.net)     -   Decentralized Uniswap V2 frontend using Helios light client for RPC reads
-   [Chatter](https://vaaxjx64bdwibfoc6nwxkelica3lfppxwilmws4s4bskfknkr7ra.vastrum.net)     -   No-metadata private messaging
-   [Concord](https://x647757zpbejyzxcw7ruqcju32otdmi7vphrg36vhhzglkjccaqq.vastrum.net)     -   Decentralized Discord
-   [Concourse](https://xv6edwxtsjtlujz2z7hgbkkshwx2im6bbvx3nxzyczumsnhwddrq.vastrum.net)   -   Decentralized Discourse
-   [Letterer](https://yozq5azfm26qi3vceclwz57fg2727yhqi6ccha5khhnp2uepqj7a.vastrum.net)    -   Decentralized Google Docs
-   [Mapper](https://lzdtxcpp6ivwje55o74dugj7f4vie6qzrsp6kybqyi7ofo3yt75q.vastrum.net)      -   Decentralized Google Maps
-   [Blocker](https://d66m4cniuqbgkeuetyvcbkfqfutt3qd3hdxdby2tlqifbt3otctq.vastrum.net)     -   Blockchain explorer for Vastrum, hosted on Vastrum


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

